# Time-Release Module

This pallet is a fork of the [ORML-vesting]( [vesting](https://github.com/open-web3-stack/open-runtime-module-library/tree/master/vesting)).

## Overview

Time-release module provides a means of scheduled balance freeze on an account. It uses the *graded release* way, which thaws a specific amount of balance every period of time, until all balance thawed.

### Release Schedule

The schedule of a release on hold is described by data structure `ReleaseSchedule`: from the block number of `start`, for every `period` amount of blocks, `per_period` amount of balance would thawed, until number of periods `period_count` reached. Note in release schedules, *time* is measured by block number. All `ReleaseSchedule`s under an account could be queried in chain state.
