# Notes on commands for doing stuff

## Digital Ocean CLI

### Creating an app

```
doctl apps create --spec spec.yaml
```

### Listing running apps

```
doctl apps list
```

### Update running app (get app ID via the list command)

```
doctl apps update APP-ID --spec=spec.yaml
```