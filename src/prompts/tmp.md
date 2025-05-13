Given all the implementations for the local store and types that currently exist in logic/local_store.rs and logic/mod.rs Could you try to architect out what the third component of this design would look like?

This is a program for batch processing pdf's for processing cheaply. It consists of primarially 3 components.

1. An API that takes in requests to process pdfs either provided in binary from a multipart form input or from an S3 URI that you would have to download. This API wouldnt consume the PDF's directly, it would instead add it to queue and intermediate file storage to get processed by a worker later. Whenever a user asks for an update it can read from the intermediate storage and task queue and give it the information.

2. An form for storing intermediate state that would handle the file storage for documents and the queue and metadata retrieval

3. A worker task that will take things from the intermediate task storage and task queue, update the state of the PDF in the DB from waiting to processing and then process the pdf, update everything. And continuously grab the next document on the stack.



I finished writing components 1 and 2. Could you go ahead and architect out the code for 3.  Don't forget to start and initialize the workers in the main function.

Keep a living diary of your thoughts at prompts/llm_thoughts.md as you apply stuff and figure it out.

Before you finish your task run ` RUSTFLAGS="-A warnings" cargo check --message-format=short` (Some optimisations to weed out a bunch of unneded tokens) to make sure you havent made any mistakes. Also try to avoid modifying any code that isnt absolutely essential to implement your feature.

Also you can look up documentation for popular rust libraries like tokio, serde and axum by using the context7 tool, its support on less popular libraries is limited unfortunately. Whenever you are stuck with some inscrutable errors, it can be helpful to look up examples to see how the code should be structured.

