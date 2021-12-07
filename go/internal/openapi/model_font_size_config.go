/*
 * Svix API
 *
 * Welcome to the Svix API documentation!  Useful links: [Homepage](https://www.svix.com) | [Support email](mailto:support+docs@svix.com) | [Blog](https://www.svix.com/blog/) | [Slack Community](https://www.svix.com/slack/)  # Introduction  This is the reference documentation and schemas for the [Svix webhook service](https://www.svix.com) API. For tutorials and other documentation please refer to [the documentation](https://docs.svix.com).  ## Main concepts  In Svix you have four important entities you will be interacting with:  - `messages`: these are the webhooks being sent. They can have contents and a few other properties. - `application`: this is where `messages` are sent to. Usually you want to create one application for each of your users. - `endpoint`: endpoints are the URLs messages will be sent to. Each application can have multiple `endpoints` and each message sent to that application will be sent to all of them (unless they are not subscribed to the sent event type). - `event-type`: event types are identifiers denoting the type of the message being sent. Event types are primarily used to decide which events are sent to which endpoint.   ## Authentication  Get your authentication token (`AUTH_TOKEN`) from the [Svix dashboard](https://dashboard.svix.com) and use it as part of the `Authorization` header as such: `Authorization: Bearer ${AUTH_TOKEN}`.  <SecurityDefinitions />   ## Code samples  The code samples assume you already have the respective libraries installed and you know how to use them. For the latest information on how to do that, please refer to [the documentation](https://docs.svix.com/).   ## Cross-Origin Resource Sharing  This API features Cross-Origin Resource Sharing (CORS) implemented in compliance with [W3C spec](https://www.w3.org/TR/cors/). And that allows cross-domain communication from the browser. All responses have a wildcard same-origin which makes them completely public and accessible to everyone, including any code on any site. 
 *
 * API version: 1.4
 */

// Code generated by OpenAPI Generator (https://openapi-generator.tech); DO NOT EDIT.

package openapi

import (
	"encoding/json"
)

// FontSizeConfig struct for FontSizeConfig
type FontSizeConfig struct {
	Base *int32 `json:"base,omitempty"`
}

// NewFontSizeConfig instantiates a new FontSizeConfig object
// This constructor will assign default values to properties that have it defined,
// and makes sure properties required by API are set, but the set of arguments
// will change when the set of required properties is changed
func NewFontSizeConfig() *FontSizeConfig {
	this := FontSizeConfig{}
	return &this
}

// NewFontSizeConfigWithDefaults instantiates a new FontSizeConfig object
// This constructor will only assign default values to properties that have it defined,
// but it doesn't guarantee that properties required by API are set
func NewFontSizeConfigWithDefaults() *FontSizeConfig {
	this := FontSizeConfig{}
	return &this
}

// GetBase returns the Base field value if set, zero value otherwise.
func (o *FontSizeConfig) GetBase() int32 {
	if o == nil || o.Base == nil {
		var ret int32
		return ret
	}
	return *o.Base
}

// GetBaseOk returns a tuple with the Base field value if set, nil otherwise
// and a boolean to check if the value has been set.
func (o *FontSizeConfig) GetBaseOk() (*int32, bool) {
	if o == nil || o.Base == nil {
		return nil, false
	}
	return o.Base, true
}

// HasBase returns a boolean if a field has been set.
func (o *FontSizeConfig) HasBase() bool {
	if o != nil && o.Base != nil {
		return true
	}

	return false
}

// SetBase gets a reference to the given int32 and assigns it to the Base field.
func (o *FontSizeConfig) SetBase(v int32) {
	o.Base = &v
}

func (o FontSizeConfig) MarshalJSON() ([]byte, error) {
	toSerialize := map[string]interface{}{}
	if o.Base != nil {
		toSerialize["base"] = o.Base
	}
	return json.Marshal(toSerialize)
}

type NullableFontSizeConfig struct {
	value *FontSizeConfig
	isSet bool
}

func (v NullableFontSizeConfig) Get() *FontSizeConfig {
	return v.value
}

func (v *NullableFontSizeConfig) Set(val *FontSizeConfig) {
	v.value = val
	v.isSet = true
}

func (v NullableFontSizeConfig) IsSet() bool {
	return v.isSet
}

func (v *NullableFontSizeConfig) Unset() {
	v.value = nil
	v.isSet = false
}

func NewNullableFontSizeConfig(val *FontSizeConfig) *NullableFontSizeConfig {
	return &NullableFontSizeConfig{value: val, isSet: true}
}

func (v NullableFontSizeConfig) MarshalJSON() ([]byte, error) {
	return json.Marshal(v.value)
}

func (v *NullableFontSizeConfig) UnmarshalJSON(src []byte) error {
	v.isSet = true
	return json.Unmarshal(src, &v.value)
}


