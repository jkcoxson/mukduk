# Mukduk Proxy

Stupid simple proxy for http servers that kills processes after a certain amount of time.

The motivation behind the project was that stable diffusion takes a lot of resources while idle, and I wanted to have a simple way to kill the process when nobody was using it.

## Building

`cargo build`

## Usage

Generate a default config file
`mukduk write`

Edit the config file
Run the config
`mukduk filename`
