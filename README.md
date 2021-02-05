# CPI Sync (cpisync.exe)

CPI Sync is a command line tool that lets you download and extract SAP CPI packages & artifacts to your local directories.

It has two main configurations that are documented below:

- Command line arguments
- Config file in JSON format

## Quick Start Guide

1. Download it from GitHub releases.
2. Create an empty directory, and put `cpisync.exe` inside the directory
3. Just copy the example config below, change the fields. Using the JSON create `cpi-config.json` file inside the same directory.
4. You can double-click on `cpisync.exe` and it will ask for the password while running.
   1. Alternative: You can start `cmd` or PowerShell, set the environment variable `CPI_PASSWORD` and then `cpisync`
      1. For Windows cmd.exe: `set CPI_PASSWORD=yourpass`
      2. For Windows Powershell: `$env:CPI_PASSWORD="yourpass"`
      3. For Linux: `export CPI_PASSWORD=yourpass`
   2. Alternative: See "Recommended Credentials: OAuth"

### Example Config

```json
{
  "cpisync": "0.1.0",

  "tenant": {
    "management_host": "change-tmn.hci.eu1change.hana.ondemand.com",
    "credential": {
      "s_user": {
        "username": "S000change",
        "password_environment_variable": "CPI_PASSWORD"
      }
    }
  },
  "packages": [
    { "id": "TrainingPackage" },
    { "id": "eDocumentElectronicInvoicingforYou" },
    { "id": "MYPACKAGE", "local_dir": "my_package_to_different_directory" }
  ]
}
```

## No clear-text password please!

You may notice there is no field called `password` and the tool will give error if it encounters one. That is a feature to prevent clear-text passwords. Current options are interactive or environment variable.

This feature makes the tool harder to use, but I think it worths the effort. And we can find both secure & more convenient solutions in the future.

## CI/CD usage

There are two ideas here:

- You can pass credential secrets via environment variables
- Use command argument `--no-input`

## Recommended Credentials: OAuth

Create an OAuth client for your tenant. Use `oauth_client_credentials` object under `credential` for the configuration.

### Example Config Part for OAuth

```json
{
  "credential": {
    "oauth_client_credentials": {
      "client_id": "CPISyncAuthClientAPI",
      "client_secret_environment_variable": "CPI_PASSWORD",
      "token_endpoint_url": "https://oauthasservices-000change000.hana.ondemand.com/oauth2/api/v1/token"
    }
  }
}
```

## Updates

When you download a new version of the tool. Schema version will be updated and you may need to change version like `"cpisync": "0.1.0"` , preferably after checking the documentation!

There may be occasional breaking changes on the format, advice & feedback from the community will play a big role.

## Reference

### Command Line Arguments Reference

```
USAGE:
    cpisync.exe [FLAGS] [OPTIONS]

FLAGS:
    -h, --help        Prints help information
        --no-input    Disable features that require user input
    -V, --version     Prints version information

OPTIONS:
    -c, --config <config>    [default: ./cpi-sync.json]
```

### JSON Config File Reference

You can inspect `config.schema.json` under `resources`. You can use a tool like ["JSON Schema Faker"](https://json-schema-faker.js.org/) to get more ideas about your options. Just paste the schema and click generate a few times!

You can also use ["JSON Schema Validator"](https://www.jsonschemavalidator.net/) if you get too many errors on your config file.
