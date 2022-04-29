// Redis queue implementation
///
/// This implementation uses the following data structures:
/// - A "tasks to be processed" queue - which is what the consumer listens to for tasks.
///     AKA: Main
/// - A "tasks currently processing" queue - which are for tasks that are currently being handled.
///     AKA: Processing
/// - A ZSET for delayed tasks with the sort order being the time-to-be-delivered
///     AKA: Delayed
///
/// - Tasks in the queues are prefixed with a ksuid so that we can know the timestamp of when they
/// should be executed.
///
/// The implementation spawns an additional worker that monitors both the zset delayed tasks and
/// the tasks currently processing. It monitors the zset task set for tasks that should be
/// processed now, and the currently processing queue for tasks that have timed out and should be
/// put back on the main queue.
use std::time::Duration;

use axum::async_trait;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use chrono::Utc;
use redis::{
    streams::{StreamPendingReply, StreamRangeReply, StreamReadOptions, StreamReadReply},
    AsyncCommands, Cmd, RedisResult, RedisWrite, ToRedisArgs,
};
use svix_ksuid::*;
use tokio::time::sleep;

use crate::error::{Error, Result};

use super::{
    QueueTask, TaskQueueConsumer, TaskQueueDelivery, TaskQueueProducer, TaskQueueReceive,
    TaskQueueSend,
};

// FIXME: Change unwraps to have our own error type for the queue module entirely

// Queue name constants
const MAIN: &str = "svix_queue_main";
const DELAYED: &str = "svix_queue_delayed";

// Consumer group name constant and counsumer group consumer name constant
const WORKERS_GROUP: &str = "svix_workers";
const WORKER_CONSUMER: &str = "svix_worker";

/// After this limit a task should be taken out of the processing queue and rescheduled
const TASK_VALIDITY_DURATION: Duration = Duration::from_secs(45);

/// Stream ID argument constants
const GENERATE_STREAM_ID: &str = "*";
const LISTEN_STREAM_ID: &str = ">";

pub async fn new_pair(
    pool: Pool<RedisConnectionManager>,
) -> (TaskQueueProducer, TaskQueueConsumer) {
    // Create the stream and consumer group for the MAIN queue should it not already exist
    {
        let mut conn = pool
            .get()
            .await
            .expect("Error retreiving connnection from Redis pool");

        let consumer_group_resp: RedisResult<()> =
            conn.xgroup_create_mkstream(MAIN, WORKERS_GROUP, 0i8).await;

        // The BUSYGROUP error is acceptable here as it just means the stream and consumer group already
        // exist and nothing has been done
        if let Err(e) = consumer_group_resp {}

        // The consumer in the consumer group is created automatically when used, so it does not have
        // to be created via `XGROUP CREATECONSUMER` command here
    }

    // Spawn a background thread handling timed out messages in the main queue and delayed messages to
    // add to the main queue.
    tokio::spawn({
        let pool = pool.clone();
        async move {
            // FIXME: enforce we only have one such worker (locking)
            let batch_size: isize = 50;
            let task_validity_duration =
                chrono::Duration::from_std(TASK_VALIDITY_DURATION).unwrap();

            loop {
                let mut pool = pool.get().await.unwrap();
                let timestamp = Utc::now().timestamp();
                let keys: Vec<String> = pool
                    .zrangebyscore_limit(DELAYED, 0isize, timestamp, 0isize, batch_size)
                    .await
                    .unwrap();
                if !keys.is_empty() {
                    // FIXME: needs to be a transaction
                    let keys: Vec<(String, String)> =
                        pool.zpopmin(DELAYED, keys.len() as isize).await.unwrap();
                    let keys: Vec<String> = keys.into_iter().map(|x| x.0).collect();
                    for key in keys {
                        let mut parts = key.split('|');
                        let _: () = pool
                            .xadd(
                                MAIN,
                                GENERATE_STREAM_ID,
                                &[(b"data", parts.nth(1).unwrap())],
                            )
                            .await
                            .unwrap();
                    }
                } else {
                    // Wait for half a second before attempting to fetch again if nothing was found
                    sleep(Duration::from_millis(500)).await;
                }

                // Every iteration here also check whether the processing queue has items that should
                // be picked back up
                let keys: StreamPendingReply = Cmd::xpending(MAIN, WORKERS_GROUP)
                    .arg("IDLE")
                    .arg(45000i16)
                    .query_async(&mut *pool)
                    .await
                    .unwrap();

                // If there are keys older than 45s, it means they should be reprocessed. `XCLAIM`
                // and all don't work very well here, so we just acknowledge the message and
                // reinsert it into the MAIN queue
                //
                // FIXME: Will keys less than 45s leak through this way?
                if let StreamPendingReply::Data(data) = keys {
                    let keys: StreamRangeReply =
                        pool.xrange(MAIN, data.start_id, data.end_id).await.unwrap();
                    for key in keys.ids {
                        let _: RedisResult<()> = pool.xack(MAIN, WORKERS_GROUP, &[key.id]).await;

                        if let Some(redis::Value::Data(data)) = key.map.get("data") {
                            let _: RedisResult<()> =
                                pool.xadd(MAIN, GENERATE_STREAM_ID, &[("data", data)]).await;
                        }
                    }
                }
            }
        }
    });

    (
        TaskQueueProducer(Box::new(RedisQueueProducer { pool: pool.clone() })),
        TaskQueueConsumer(Box::new(RedisQueueConsumer { pool })),
    )
}

/// Enum for the LEFT | RIGHT args used by some commands
pub enum Direction {
    Left,
    Right,
}

impl ToRedisArgs for Direction {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + RedisWrite,
    {
        let s: &[u8] = match self {
            Direction::Left => b"LEFT",
            Direction::Right => b"RIGHT",
        };
        out.write_arg(s);
    }
}

fn to_redis_key(delivery: &TaskQueueDelivery) -> String {
    format!(
        "{}|{}",
        delivery.id,
        serde_json::to_string(&delivery.task).unwrap()
    )
}

fn from_redis_key(key: &str) -> TaskQueueDelivery {
    // Get the first delimiter -> it has to have the |
    let pos = key.find('|').unwrap();
    let id = (&key[..pos]).to_string();
    let task = serde_json::from_str(&key[pos + 1..]).unwrap();
    TaskQueueDelivery { id, task }
}

#[derive(Clone)]
pub struct RedisQueueProducer {
    pool: Pool<RedisConnectionManager>,
}

#[async_trait]
impl TaskQueueSend for RedisQueueProducer {
    async fn send(&self, task: QueueTask, delay: Option<Duration>) -> Result<()> {
        let mut pool = self
            .pool
            .get()
            .await
            .expect("Error retreiving connection from pool");

        let timestamp = delay.map(|delay| Utc::now() + chrono::Duration::from_std(delay).unwrap());

        if let Some(timestamp) = timestamp {
            let delivery = TaskQueueDelivery::new(task, Some(timestamp));
            let key = to_redis_key(&delivery);
            let _: () = pool
                .zadd(DELAYED, key, timestamp.timestamp())
                .await
                .expect("Could not add task to delayed queue");
        } else {
            // XXX: Check argument format
            let _: () = redis::cmd("XADD")
                .arg(MAIN)
                .arg(GENERATE_STREAM_ID)
                .arg("data")
                .arg(serde_json::to_vec(&task).expect("Serialization error"))
                .query_async(&mut *pool)
                .await
                .expect("Could not add task to main queue");
        }
        tracing::trace!("RedisQueue: event sent > (delay: {:?})", delay);
        Ok(())
    }

    async fn ack(&self, delivery: TaskQueueDelivery) -> Result<()> {
        let mut pool = self
            .pool
            .get()
            .await
            .expect("Error retreiving connection from pool");
        let processed: u8 = pool
            .xack(MAIN, WORKERS_GROUP, &[delivery.id])
            .await
            .expect("Could not acknowledge task");

        if processed != 1 {
            tracing::warn!(
                "Expected to remove 1 from the list, removed {} for {}|{}",
                processed,
                delivery.id,
                serde_json::to_string(&delivery.task).unwrap(),
            );
        }

        Ok(())
    }

    async fn nack(&self, delivery: TaskQueueDelivery) -> Result<()> {
        // FIXME: do something else here?
        let mut pool = self
            .pool
            .get()
            .await
            .expect("Error retreiving connection from pool");
        let _: u8 = pool
            .xack(MAIN, WORKERS_GROUP, &[delivery.id])
            .await
            .expect("Could not acknowledge task");

        tracing::error!(
            "Failed processing msg: {}|{}",
            delivery.id,
            serde_json::to_string(&delivery.task).unwrap()
        );

        Ok(())
    }

    fn clone_box(&self) -> Box<dyn TaskQueueSend> {
        Box::new(self.clone())
    }
}

pub struct RedisQueueConsumer {
    pool: Pool<RedisConnectionManager>,
}

#[async_trait]
impl TaskQueueReceive for RedisQueueConsumer {
    async fn receive(&mut self) -> Result<TaskQueueDelivery> {
        let mut pool = self
            .pool
            .get()
            .await
            .expect("Error retreiving connection from Redis pool");

        let resp: StreamReadReply = pool
            .xread_options(
                &[MAIN],
                &[LISTEN_STREAM_ID],
                &StreamReadOptions::default().group(WORKERS_GROUP, WORKER_CONSUMER),
            )
            .await
            // FIXME: Error handling or at least an error message
            .unwrap();

        // FIXME: What if it returns no responses
        let id = resp.keys[0].ids[0].id;
        let map = resp.keys[0].ids[0].map;

        let task: QueueTask = if let Some(redis::Value::Data(data)) = map.get("data") {
            serde_json::from_slice(data).expect("Invalid QueueTask")
        } else {
            panic!("No QueueTask associated with key");
        };

        tracing::trace!("RedisQueue: event recv <");
        Ok(TaskQueueDelivery { id, task })
    }
}
