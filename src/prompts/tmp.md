There is some documentation for how to make tracing ingest to OTEL successfully in :
/home/nicole/Documents/mycorrhizae/crimson/src/prompts/references/otel_tracing_docs.md

There is also another file for how to export to an oltp endpoint:
/home/nicole/Documents/mycorrhizae/crimson/src/prompts/references/otel_export_docs.md


Could you look at these docs and figure out how to make the tracing/logging code in here successfully ingest to an otel endpoint. The endpoint should be given by an enviornment variable OTEL_EXPORTER_OTLP_ENDPOINT and export it using grpc.


There are also example projects for how to get logs and tracing working at 
/home/nicole/Documents/mycorrhizae/crimson/src/prompts/references/logs-basic
/home/nicole/Documents/mycorrhizae/crimson/src/prompts/references/tracing-grpc
