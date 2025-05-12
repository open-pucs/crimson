# What should you do?

Given the below description of the library, go ahead and architect out what the intermediate storage code should look like and specifically what libraries and dependancies should I use.

In this report could you also write down any necessary types that help deal with the intermediate storage (This already partially exists with the FileLocation enum, and more types in this format would be really helpful.)


Once done write your output to /prompts/intermediate_storage.md


# What is this library.

This is a program for batch processing pdf's for processing cheaply. It consists of primarially 3 components.

1. An API that takes in requests to process pdfs either provided in binary from a multipart form input or from an S3 URI that you would have to download. This API wouldnt consume the PDF's directly, it would instead add it to queue and intermediate file storage to get processed by a worker later. Whenever a user asks for an update it can read from the intermediate storage and task queue and give it the information.

NOTE: This is essentially complete, the only parts that need to be added are in /src/logic/mod.rs


2. A worker task that will take things from the intermediate task storage and task queue, update the state of the PDF in the DB from waiting to processing and then process the pdf, update everything. And continuously grab the next document on the stack.


3. An form for storing intermediate state that would handle the file storage for documents and the queue and metadata retrieval. I think it should have two modes:

- A mode using an s3 compatible API for file storage, and redis for storage of file metadata and the document processing queue.
- A local only mode that would use the file system on device, and some kind of in memory database.


All the types that currently exist for this are in 

/src/types/mod.rs



# S3 module code.


