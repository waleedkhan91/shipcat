# sample manifests

This folder contains a sample manifest repository setup for a kubernetes cluster of version >= 1.13.

```sh
├── shipcat.conf
├── charts
│   └── base/...
└── services
    ├── blog
    │   └── manifest.yml
    └── webapp
        └── manifest.yml
```

## Requirements
A cluster defined in `shipcat.conf` (two example environments defined therein), reachable [through dns](https://github.com/clux/kube-rs/issues/153).

More examples in the `Makefile`.

### Example 1: Minikube
A [minikube](https://github.com/kubernetes/minikube) [none driver](https://minikube.sigs.k8s.io/docs/reference/drivers/none/), running in the `apps` namespace:

```sh
sudo -E minikube start --driver=none --kubernetes-version v1.15.8 --extra-config kubeadm.ignore-preflight-errors=SystemVerification
kubectl config set-context --cluster=minikube --user=minikube --namespace=apps minikube
kubectl create namespace apps
```

### Example 2: Kind
A [kind](https://github.com/kubernetes-sigs/kind) cluster in the default namespace:

```sh
kind create cluster --name shipcat
```

## Installation
The only thing you need to install are the CRDs that shipcat define, which can be done with:

```sh
shipcat cluster crd install
```

## Local Exploration
You can use `shipcat` at the root of this folder, or anywhere else if you point `SHIPCAT_MANIFEST_DIR` at it. Here are some examples:

Check completed manifest:

```sh
shipcat values blog
```

Check generated kube yaml:

```sh
shipcat template blog
```

Apply it to your cluster:

```sh
shipcat apply blog
```

Diff the template against what's running:

```sh
shipcat diff blog
```

Note that the `webapp` example requires external dependencies which you can configure with `make integrations` or follow along below.


## Vault Integration
Secrets are resolved from `vault`, so let's install a sample backend using `docker`:

```sh
sudo docker run --cap-add=IPC_LOCK -e 'VAULT_ADDR=http://127.0.0.1:8200' -e 'VAULT_TOKEN=myroot' -e 'VAULT_DEV_ROOT_TOKEN_ID=myroot' -e 'VAULT_DEV_LISTEN_ADDRESS=0.0.0.0:8200' -p 8200:8200 -d --rm --name vault vault:0.11.3
export VAULT_ADDR=http://127.0.0.1:8200
export VAULT_TOKEN=myroot
vault secrets disable secret
vault secrets enable -version=1 -path=secret kv
```

## Install a database
The `webapp` service relies on having a database. If you want to supply your own working `DATABASE_URL` in vault further down, you can do so yourself. Here is how to do it with [helm 3](https://github.com/helm/helm/releases):

```sh
helm repo add bitnami https://charts.bitnami.com/bitnami
helm install postgresql -n=webapp-pg bitnami/postgresql --version 12.1.4 --set global.postgresql.auth.postgresPassword=pw,global.postgresql.auth.database=webapp,primary.persistence.enabled=false
```

Then we can write the external `DATABASE_URL` for `webapp`:

```sh
vault write secret/example/webapp/DATABASE_URL value=postgres://postgres:pw@postgresql.webapp-pg:5432/webappvault write secret/example/webapp/DATABASE_URL value=postgres://postgres:pw@postgresql.webapp-pg:5432/webapp
```

You can verify that `shipcat` picks up on this via: `shipcat values -s webapp`.

### Slack integrations
Both the `shipcat apply` and `shipcat cluster` commands will pick up on some slack evars to be able send result notifications:

```sh
export SLACK_SHIPCAT_HOOK_URL=https://hooks.slack.com/services/.....
export SLACK_SHIPCAT_CHANNEL=#test
```

If these are unset, then there are still upgrade results visible on the status object:

```sh
kubectl get sm blog -oyaml | yq ".status" -y
```

## Cluster reconcile
Now that all our dependencies are set up; we can ensure our cluster is up-to-date with our repository:

```sh
shipcat cluster crd reconcile
```

This will install all the necessary custom resource definitions into kubernetes, then install the `shipcatmanifest` instances of `blog` and `webapp` (in parallel).

To garbage collect a release, you can delete its `shipcatmanifests`:

```sh
kubectl delete shipcatmanifest webapp blog
```

Re-running `reconcile` after doing so will reinstall the services.

After having reconciled a cluster, you can then run individual `shipcat apply webapp` commands manually.

## Checking it works
You can hit your api by port-forwarding to it:

```sh
kubectl port-forward deployment/webapp 8000
curl -s -X POST http://0.0.0.0:8000/posts -H "Content-Type: application/json" \
  -d '{"title": "hello", "body": "world"}'
curl -s -X GET "http://0.0.0.0:8000/posts/1"
```

## Security
Ensure the current commands are run before merging into a repository like this folder:

```sh
shipcat config verify
shipcat verify
shipcat cluster check
shipcat secret verify-region -r minikube --changed=blog,webapp
shipcat template webapp | kubeval -v 1.13.8 --strict
```
