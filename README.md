# About:
A sample cli for getting most recent logs from cloudwatch

## The problem:
- Often cloudwatch will show logs with delay of few minutes, which isn't productive & not cool for developer experience
- To see the logs we have to login through UI or use aws cli with lots of inuputs, one is log group which generally has some uuid name which is hard to remember

## Solution:
for now (until packed and optimized in future commits) can use this cli tool by using cargo (app is written in Rust),
result is quick, in terminal (which is very convenient to grep/ripgrep things or maybe use pipes + jq etc

aws creds (sso profile name),region and log group name are stored in .env file, so no more long mumbo jumbo cli arguments
 > Actually there is clap under the hood, bur for now, basic ```cargo run``` shall be more than enough 

## How to Install:

1) clone the repo:
```bash
git clone git@github.com:hnariman/logster.git 
```
2) cd into newly created directory:
```bash
cd logster
```

3) use .env template to create your own local .env (it's in git igore by default for your convenience)
```bash
cp .env.sample .env
```

4) update .env file with your data:
- AWS_REGION
- AWS_PROFILE - this one tested with profile name generated with 
```bash
aws configure sso
``` 
[AWS documentation](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-sso.html#sso-configure-profile-token-auto-sso)

- LOG_GROUP - this is the log group you would search in CloudWatch dropdown,
just find it once, copy paste into env and no need to keep this in clipboard

5) run with cargo (build yourself solution, ok for open source, will build and release it in future versions)

```bash
cargo run
```

6) there are options for arguments (as timeframe for lookback in format "15m", "2h" etc, results limit which is MAX 10_000 by default etc)
```bash
cargo run -- --help
```

