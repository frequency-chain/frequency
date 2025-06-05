# AGENT GUIDELINES
- This document provides guidelines for contributing to the project.
- Please ensure that your contributions adhere to the project's coding standards and style guidelines.

## How to Contribute
- Fork the repository and create a new branch for your feature or bug fix.
- Make your changes and commit them with clear, descriptive commit messages.
- Submit a pull request with a detailed description of your changes.

## Code Review Process
- All contributions will be reviewed by the project maintainers.
- Feedback will be provided, and changes may be requested before merging.

## Testing Your Changes
- Ensure that your changes are covered by tests.
- Run the test suite to verify that all tests pass before submitting your pull request.

## Documentation
- Update the documentation to reflect any changes made to the codebase.
- Ensure that the documentation is clear and easy to understand.

## Issues and Bug Reports
- If you find a bug, please open an issue in the repository.
- Provide as much detail as possible, including steps to reproduce the issue and any relevant logs or screenshots.

## Coding Standards
- Follow the project's coding standards and style guidelines.
- Use meaningful variable and function names.
- Write clear and concise comments where necessary.

## License
- By contributing to this project, you agree that your contributions will be licensed under the project's license.

## Dev Environment Tips
- Set up your development environment according to the project's guidelines.
- Use the provided scripts or tools to manage dependencies and run the project locally.
- Regularly pull the latest changes from the main branch to keep your branch up to date.

## Testing Instructions
- Find the CI plan in the .github directory. `verify-pr-commit.yml` is the main CI plan that runs on every PR.
- First run `make check` to ensure that cargo is installed and the project is set up correctly.
- Run the build to ensure that the project compiles without errors.
    ```bash
    make build-no-relay
    ```
- Rust builds can take up to 20 minutes, so be patient.
- Run the unit test suite using the provided commands. Network access is not required for these tests.
    ```bash
    make test
    ```
- Run the e2e test suite using the provided commands. Network access is not required for these tests, only that you have a local node running.
    ```bash
    make e2e-tests
    ```
- Run the linter to check for code style issues.
    ```bash
    make lint
    ```
- Run the formatter to ensure code style consistency.
    ```bash
    make lint-fix
    ```

## PR Instructions
Title Format: `[AGENT] <short description>`
