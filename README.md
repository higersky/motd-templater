# MOTD Templater: Creating Dynamic SSH Login Banners with Template Strings

This is an SSH Login Banner Generator for Linux. With its easy-to-use template-based approach, you can easily create dynamic and informative Message of the Day (MOTD) messages to greet users upon login.

## Installation: Building from Source

To get started, follow these steps to build it from source:

1. Download and install the latest version of the Rust compiler from the official website [rustup.rs](https://rustup.rs/). 
    
2. Clone the repository and build the application using Cargo.

```bash
$ git clone https://github.com/higersky/motd-templater
```

3. Once the compilation is complete, copy the generated executable to a folder within your system's PATH for easy access:

```bash
$ sudo cp target/release/motd-templater /usr/local/bin
```

## Usage

MOTD Templater allows you to generate MOTD messages by parsing template files. These templates can be utilized in various MOTD scripts to display customized login banners.

The program expects you to provide the path to a template file as an argument. For example, let's assume you have a template file named `sysinfo.motd-template` located in the `sample` folder. After executing the command, MOTD Templater will process the template file and replace template expressions with their actual values. It will then display the generated MOTD message in the terminal.

```bash
$ motd-templater sample/sysinfo.motd-template
> 24 cores Server ubuntu with Sky @ Thu Jul  6 20:00:00 CST 2023
  System Load   : 3.64  Physical Memory : 42.8%
  Disk Usage    : 56%   Swap Usage      : 2.7
  Sessions      : 3     CUDA Version    : 12.1.1
```

## Template Syntax

The template file utilized by MOTD Templater follows a braces-based format string syntax. This syntax allows you to include dynamic content and variable substitutions in your MOTD messages. Let's take a closer look at the template syntax:

1.  Defining Custom Variables (Optional):
    
    At the beginning of the template file, you can assign custom variables using shell commands. Each assignment should be declared on a separate line. Here's an example:
    
    ```text
    @{
       nickname := echo Sky
       date := date
       users := echo $((`users | wc -w` + 1))
    
       env root_usage = $root_disk_usage
       env memory_usage = $memory_usage
       env swap_usage = $swap_usage
       env cpu_cores = $cpu_cores
       env load = $load1
       warn := bash ./test/warn.sh
    }
    ```
    
    You can assign variables using the `:=` syntax with shell commands. Use the `env` keyword to assign environment variables with built-in variables. These variables can be referenced later in the template.
    
2.  Creating MOTD Content:
    
    After defining variables, you can write the contents of your MOTD message. MOTD Templater will replace template expressions with their corresponding values. Here's an example:
        
    ```text
    > {$cpu_cores} cores Server {$hostname:underline} with {nickname:bold:underline} @ {date}
      System Load   : {$load5}   Physical Memory : {$memory_usage:percent:warn_color}
      Disk Usage    : {$root_disk_usage:percent:warn_color}   Swap Usage    : {$swap_usage}
      Sessions      : {users}   CUDA Version    : {$cuda_version}
    {warn}
    ```
    
    In the example above, we used variables like `{$cpu_cores}` to insert their values into the MOTD message. You can also use built-in variables starting with `$`, such as `$login_user`, `$hostname`, `$memory_usage`, and more (refer to `src/handlers.rs` for a complete list of built-in variables). Additionally, you can apply modifiers like `underline`, `bold`, `percent`, and `warn_color` to format and style the text.
    

## Built-in Variables: Accessing System Information

MOTD Templater provides a set of built-in variables that give you access to various system information. These variables can be used within your template files to display relevant data. Here's a list of built-in variables:

*   `$login_user`: Get the login username of the current SSH session.
*   `$load1` / `$load5` / `$load15`: System load values from `/proc/loadavg`.
*   `$hostname`: The system hostname.
*   `$kernel_version`: The Linux kernel version.
*   `$memory_usage`: RAM usage percentage.
*   `$swap_usage`: Swap usage percentage.
*   `$cpu_cores`: The number of CPU cores in the system.
*   `$root_disk_usage`: Disk usage percentage of the mount point `/`.
*   `$data_disk_usage`: Disk usage percentage of the mount point `/data`.
*   `$cuda_version`: Default CUDA toolkit version.

You can modify `src/handler.rs` to add more built-in variables.

## Modifiers: Adding Style and Formatting

To enhance the visual appeal of your MOTD messages, MOTD Templater offers various modifiers that allow you to apply styles and formatting. Here are some of the available modifiers:

*   `underline`: Adds underlines to the text.
*   `bold`: Makes the text bold.
*   `percent`: Appends a `%` character.
*   `warn_color`: Colorizes a usage percentage value between 0 and 100.

By applying these modifiers to your template expressions, you can make your MOTD messages more visually appealing and easy to read.

That's it! You are now equipped with the knowledge to create engaging and dynamic MOTD messages using MOTD Templater. 
