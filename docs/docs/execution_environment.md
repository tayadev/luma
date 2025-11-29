# Execution Environment

Luma can run in multiple environments, each with its own capabilities and limitations.

Luma can be run from its own VM, or embedded in other applications.
The VM can run in a variety of host environments, thus providing different levels of access to system resources.

Given this flexibility some "basic" functionality needs to be explicitly imported based on the execution environment.

Thus Luma's `import()` function needs to be provided by the host environment and may not be available in all environments or behave differently.