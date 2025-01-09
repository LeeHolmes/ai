# ai.exe
## A console interface for Azure Open AI

While AI has blossomed in capability, it is usually locked behind a web interface. And that prevents you from easily using it for automation and ad-hoc tasks. Ai.exe solves this problem - a simple tool to let you run AI from the console.

![image](https://github.com/user-attachments/assets/e350204f-6b85-40c0-b051-0cd669edbb7e)

## First lanuch

At first launch, ai.exe will prompt you for an Azure Open AI key, endpoint, and deployment name. It will store these into Windows' credential manager.

## Usage

Run `ai --help` to see the command line parameters.

```
[C:\temp]
PS:50 > ai --help
AI Command Line Tool

USAGE:
    ai.exe [--prompt <prompt_file_or_text>] <input_file_or_text>
    ai.exe --delete-keys
    ai.exe --help

DESCRIPTION:
    A command line tool for interacting with Azure OpenAI services.

OPTIONS:
    --prompt <prompt_file_or_text>  Specify system prompt from file or direct text
                                    If not provided, defaults to general assistance
    --delete-keys                   Delete all stored credentials
    --help, -h                      Display this help message

ARGUMENTS:
    <input_file_or_text>            Input to process - either a file path or direct text

CREDENTIALS:
    The tool securely stores the following credentials:
    - Azure OpenAI API Key
    - Azure OpenAI Endpoint
    - Azure OpenAI Deployment Name

    On first launch, you will be prompted to enter these credentials.
    They will be stored securely in the system keyring for future use.
    Use --delete-keys to remove stored credentials.
```
