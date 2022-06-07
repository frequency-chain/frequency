# Delegator <-> Provider Grants

## Context and Scope

MRC enables users(read delegators) to have control over their own data. While providers can send and receive messages on behalf of users, there is need for separation of control via **delegator->provider**, **provider<->delegator** relationships in form of Permissions and Grants respectively. This document describes the design of the two types of relationships and recommendation on how to implement them.

## Problem Statement

Data Access Pattern on MRC, at-minimum, should provide ***PUBLISHER*** and ***RESTRICTED*** ```permissions``` at **delegator->provider**, as well as ***PUBLISH*** and, ***BLOCKED***, ```grants``` for specific ```schema_id``` at **provider<->delegator**.This entails users can enable specific permissions for provider to write data on their behalf, while also restricting grants to providers at schema level, rendering providers as restricted. Providers should also be able to opt into publish, on behalf of, users, or block from publication, on behalf of, at schema level. Primarily, the use case can be summarized in following way:

- **As a provider**, I would want to publish data for specific ```schema_id``` on-behalf of a delegator. Defaults to ```publish``` permissions on all schemas registered by provider on behalf of delegator.
- **As a delegator**, I would like to restrict a provider, by allowing a provider to only publish data for specific ```schema_ids``` on-behalf of me.

Note: A publish state would mean that a provider is able to publish data on behalf of a delegator on all public schemas by passing validation. While a restricted state would mean that a provider is not able to publish data on behalf of a delegator on a specific schema, would require additional validation. The default state would be restricted as provider must opt in (permissioned by user) to publish data for specific schema(s) on user's behalf before sending messages for said schema(s).

## Goals and Non-Goals

MRC is a default read only for items stored on and off chain, requiring an explicit process to control writing or publishing of messages via some permissions and grants. Some of the major goals surrounding provider permissions and grants are:

### Goals

**Opt In and Duality**: Providers should register users with MRC and delegate on behalf of them, while also specifically allowing a collection of schema(s) for which delegator provide them full publication rights. This ensures default state of providers is ***Restrict***. Delegators can also choose to restrict providers on per-schema basis by blocking them from publishing data on their behalf. This ensures default state of delegators is ***Block*** for all non provider preferred ```schema_ids```. Duality should be implemented at schema level grants.

**ToS Baked In**: As a part of this design doc, it is recommended to discuss about baking in ***ToS*** for providers and delegators as a part of permission grants by including a hash of ToS unless there is a re-delegation. Such that MRC can also act as proof of specific agreement established between a provider and a delegator.

**Time Bounded Grants**: Any grants given or revoked by a delegator (allowing provider to publish or block them for certain duration) or any grants are modified by a provider or delegator are valid for the duration of ***ToS***. This can be a control mechanism in MRC which can be a fixed number for version 1 of this implementation and be extensible via a governance mechanism, by executing an extrinsic. This also brings the question about, if not time bounded, does permissions and grants are set till they are explicitly revoked. De-delegation, definitely is an option for user to remove a provider completely from ever publishing on their behalf.

### Non-Goals

## Proposal

## Benefits and Risks

## Additional Resources
