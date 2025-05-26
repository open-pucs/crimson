There is some documentation for how to make tracing ingest to OTEL successfully in :
/home/nicole/Documents/mycorrhizae/crimson/src/prompts/references/otel_tracing_docs.md

There is also another file for how to export to an oltp endpoint:
/home/nicole/Documents/mycorrhizae/crimson/src/prompts/references/otel_export_docs.md


Could you look at these docs and figure out how to make the tracing/logging code in here successfully ingest to an otel endpoint. The endpoint should be given by an enviornment variable OTEL_EXPORTER_OTLP_ENDPOINT and export it using grpc.


This should be everything you need to complete this project, but if you get really confused you can also look at these example projects to see if your syntax is correct.
/home/nicole/Documents/mycorrhizae/crimson/src/prompts/references/logs-basic
/home/nicole/Documents/mycorrhizae/crimson/src/prompts/references/tracing-grpc


Before you finish your task run ` RUSTFLAGS="-A warnings" cargo check --message-format=short` (Some optimisations to weed out a bunch of unneded tokens) to make sure you havent made any mistakes. Also try to avoid modifying any code that isnt absolutely essential to implement your feature.
