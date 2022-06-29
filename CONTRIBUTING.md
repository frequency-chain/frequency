# CONTRIBUTING

For contributing guidelines see the [Project Liberty Contributing Guidelines](https://github.com/LibertyDSNP/meta/blob/main/CONTRIBUTING.md).


# Path to Beta

While we are trying to get Frequency off the ground, the work is slightly different than with a stable project.
Here are our rules of thumb
How we work while trying to get off the ground.
Expect this to change as we stabilize.

- Issues should often be MORE than one PR
  - Smaller PRs are easier to review
  - Great if your story is blocking others and a part of your story complete can unblock them
  - Consider breaking up into:
    - Core Logic
    - Addition PR Feedback
    - Benchmarks
    - Smaller side logic
    - Etc...
- PR Reviews
  - We want to get them through fast.
  -  Prefix Blocking Comments with "Blocking:" (or other indication that it is important)
  - Types of Comments:
    - Something that MUST be changed before being merged. (Mark PR review as "Request Changes")
    - Something that needs to be done before the Issue is closed, but the PR can be merged. PR Author should add it to a TODO list in the Issue.
    - Something that should be in a SEPARATE issue. PR Author creates a new issue.
    - Suggestions. Things that could be done differently, but only if the PR Author wants to.
- Internals can change much easier than externals
  - Example: Extrinsic interface changes should be vetted MORE than internal code (for now).
