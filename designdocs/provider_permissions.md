# Delegator <-> Provider Grants

## Context and Scope

MRC enables users(read delegators) to have control over their own data. While providers can send and receive messages on behalf of users, there is need for separation of control via **delegator->provider**, **provider<->delegator** relationships in form of Permissions and Grants respectively. This document describes the design of the two types of relationships and recommendation on how to implement them.

## Problem Statement

Authorization on MRC, at-minimum, should provide ***PUBLISHER*** , ***SUBSCRIBER*** and ***RESTRICTED*** level ```permissions```, as well as ***Read*** , ***Write*** and, ***Private***, level ```grants``` at specific ```schema_id``` to begin with.This entails users can enable specific permissions for provider to read and/or write data on their behalf, while while also restricting grants to providers at schema level, if they choose to restrict the delegation to providers and vice versa. For example.

- **As a provider**, I would want to publish and/or subscribe to specific data for very specific schemas. This document is centered around the design of the Permissions and Grants.
- **As an app user**, I would want authorize read/write grants to provider for specific data type. I should also be able to take specific data type private.

## Goals

## Proposal

## Benefits and Risks

## Additional Resources
