![Shipcat](shipcat.png)

[![CircleCI](https://circleci.com/gh/Babylonpartners/shipcat.svg?style=shield&circle-token=1e5d93bf03a4c9d9c7f895d7de7bb21055d431ef)](https://circleci.com/gh/Babylonpartners/shipcat)

[![Docker Repository on Quay](https://quay.io/repository/babylonhealth/kubecat/status?token=6de24c74-1576-467f-8658-ec224df9302d "Docker Repository on Quay")](https://quay.io/repository/babylonhealth/kubecat)

A standardisation tool sitting in front of `helm` to help control deployments on `kubernetes` via `shipcat.yml` manifest files.

Lives [on your ship](https://en.wikipedia.org/wiki/Ship%27s_cat).

## Installation
### Prebuilt
Grab a [prebuilt shipcat from github releases](https://github.com/Babylonpartners/shipcat/releases). This can be extracted to `/usr/local` or some place on your `$PATH`. Then add the line `source /usr/local/share/shipcat/shipcat.complete.sh` to `~/.bash_completion`.

### Building
To build yourself, use [rustup](https://rustup.rs/) to get latest stable rust.

```sh
rustup update stable # if build breaks on master
cargo build
ln -sf $PWD/target/debug/shipcat /usr/local/bin/shipcat
echo "source $PWD/shipcat.complete.sh" >> ~/.bash_completion
```

### Docker
The embedded `kubecat` image does come with `shipcat` + `kubectl` + `helm` + `helm diff` plugin + `kubeval`, which might be easier to use together. Bring a valid `~/.kube/config` and `VAULT_*` evars.

## Usage - Read Access
In general, define your `shipcat.yml` file in the [manifests repo](https://github.com/Babylonpartners/manifests) and make sure `shipcat validate` passes.

If you have `vault` read credentials you can validate secret existence; and generate the complete helm template:

```sh
export VAULT_ADDR=...
export VAULT_TOKEN=...

# Verify manifests secrets exist
shipcat validate babylbot --secrets
```

If you have `kubectl` read only credentials you can also create your helm values, template and diff a deployment against a current running one:

```sh
# Generate helm values for your chart
shipcat helm babylbot values

# Pass the generated values through helm template
shipcat helm babylbot template

# Diff the helm template against the live deployment
shipcat helm babylbot diff
```

Note that this requires `helm` + [helm diff](https://github.com/databus23/helm-diff) installed to work, and it will work against the region in your context (`kubectl config current-context`).

## Usage - Write Access
With rollout access (`kubectl auth can-i rollout Deployment`) you can also perform upgrades:

```sh
# helm upgrade corresponding service (check your context first)
shipcat helm babylbot upgrade
```

This requires slack credentials for success notifications.

If you have `slack` credentials, you can also use `shipcat slack` to talk to slack directly:

```sh
export SLACK_SHIPCAT_HOOK_URL=...
export SLACK_SHIPCAT_CHANNEL="#kubernetes"
shipcat slack hi slack
```

If you have jenkins credentials, you can also use `shipcat jenkins` to query for job history:

```sh
export JENKINS_URL=https://jenkins.babylontech.co.uk
export JENKINS_API_TOKEN=TOKEN_FROM_PROFILE_PAGE
export JENKINS_API_USER=eirik.albrigtsen

shipcat jenkins diagnostic-engine history
shipcat jenkins core-ruby console -n 3022 | less
```

## Docs
API documentation is autogenerated via `cargo doc`, but not hosted anywhere. It can be viewed via:

```sh
cargo doc
xdg-open target/doc/shipcat/index.html
```

Explicit guides for shipcat is available in the [doc directory](https://github.com/Babylonpartners/shipcat/tree/master/doc). In particular:

- [extending shipcat](https://github.com/Babylonpartners/shipcat/tree/master/doc/extending.md)
