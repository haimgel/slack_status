## SlackStatus - set Slack status for multiple accounts 

This tiny app allows to set Slack status to pre-defined values from a command line,
for multiple accounts at once:

```bash
 slack_status lunch
```

This will set yourself away for an hour, with an hamburger emoji and "Lunch" status text.

### How to use

1. Copy `slack_status.toml.example` to `~/.slack_status.toml` and edit it to your liking.
2. Have [rust](https://www.rust-lang.org/) installed.
3. Compile this application: `cargo build --release`
4. Copy `./target/release/slack_status` and `settings.toml` somewhere convenient. 
5. Run it!

### Caveats

I created this as a weekend exercise while learning Rust. Treat it as a homework-quality code.
