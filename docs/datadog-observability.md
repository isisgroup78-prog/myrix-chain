# Datadog observability

The deployment stack can send MYRIX Prometheus metrics and container logs to Datadog through the Datadog Agent.

## Required deployment secret

Set `DD_API_KEY` in the deployment environment. Never commit a real API key to Git.

Optional variables:

- `DD_SITE`: Datadog site, default `datadoghq.com`.
- `DD_ENV`: environment tag, default `staging`.

The Agent scrapes `http://myrix-api:3000/metrics` using the OpenMetrics integration and namespaces the metrics under `myrix.*`. It also collects Docker container logs.

Datadog documents OpenMetrics/Prometheus scraping through `conf.d/openmetrics.d/conf.yaml`.

After deployment, verify that MYRIX metrics appear in Datadog before creating alerts or dashboards.

Reference: https://docs.datadoghq.com/integrations/openmetrics/
