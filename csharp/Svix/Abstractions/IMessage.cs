﻿using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using Svix.Model;
using Svix.Models;

namespace Svix.Abstractions
{
    public interface IMessage
    {
        MessageOut Create(string appId, MessageIn message, MessageCreateOptions options,
            string idempotencyKey = default);

        Task<MessageOut> CreateAsync(string appId, MessageIn message, MessageCreateOptions options,
            string idempotencyKey = default, CancellationToken cancellationToken = default);

        MessageOut Get(string appId, string messageId, string idempotencyKey = default);

        Task<MessageOut> GetAsync(string appId, string messageId, string idempotencyKey = default,
            CancellationToken cancellationToken = default);

        List<MessageOut> List(string appId, MessageListOptions options, string idempotencyKey = default);

        Task<List<MessageOut>> ListAsync(string appId, MessageListOptions options, string idempotencyKey = default,
            CancellationToken cancellationToken = default);
    }
}