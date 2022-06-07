# Delegator <-> Provider Grants

## Context and Scope

MRC enables users(read delegators) to have control over their own data. While providers can send and receive messages on behalf of users, there is need for separation of control via **delegator->provider**, **provider<->delegator** relationships in form of Permissions and Grants respectively. This document describes the design of the two types of relationships and recommendation on how to implement them.

## Problem Statement

Data Access Pattern on MRC, at-minimum, should provide ***PUBLISHER*** and ***RESTRICTED*** ```permissions``` at **delegator->provider**, as well as ***PUBLISH*** and, ***BLOCKED***, ```grants``` for specific ```schema_id``` at **provider<->delegator**.This entails users can enable specific permissions for provider to write data on their behalf, while also restricting grants to providers at schema level, rendering providers as restricted. Providers should also be able to opt into publish, on behalf of, users, or block from publication, on behalf of, at schema level. Primarily, the use case can be summarized in following way:

- **As a provider**, I would want to publish data for specific ```schema_id``` on-behalf of a delegator. Defaults to ```publish``` permissions on all schemas.
- **As a delegator**, I would like to restrict a provider, by blocking a provider from publishing data for specific ```schema_id``` on-behalf of me.

Note: A publish state would mean that a provider is able to publish data on behalf of a delegator on all public schemas by passing validation. While a restricted state would mean that a provider is not able to publish data on behalf of a delegator on a specific schema, would require additional validation. The default state would be restricted as provider must opt in (permissioned by user) to publish data for specific schema(s) on user's behalf before sending messages for said schema(s).

## Goals

## Proposal

## Benefits and Risks

## Additional Resources
