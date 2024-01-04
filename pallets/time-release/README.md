# Time-Release Module

This pallet is a fork of the [ORML-vesting]( [vesting](https://github.com/open-web3-stack/open-runtime-module-library/tree/master/vesting)).

## Overview

The `time-release` module offers a method for scheduling a balance freeze on an account. It employs a *graded release* approach, which thaws a specific amount of balance every period of time, until all balance is thawed.

### Release Schedule

The schedule of a release on hold is described by the data structure `ReleaseSchedule`. Starting from the specified block number denoted as `start`, the schedule operates on a periodic basis. For each `period` amount of blocks, a designated `per_period` amount of balance is unfrozen. This process continues until the specified number of periods, denoted as `period_count`, is reached. It's important to highlight that in release schedules, the concept of time is measured in terms of block numbers. Accessing all `ReleaseSchedule` instances associated with an account is possible through querying the chain state.
