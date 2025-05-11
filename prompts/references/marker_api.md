     

API Endpoints

All endpoints will return immediately, and continue processing in the background.
Marker

The marker endpoint converts PDFs, spreadsheets, word documents, epub, HTML, and powerpoints to markdown. It is available at /api/v1/marker.

Here is an example request in Python:

import requests

url = "https://www.datalab.to/api/v1/marker"

form_data = {
    'file': ('test.pdf', open('~/pdfs/test.pdf', 'rb'), 'application/pdf'),
    'langs': (None, "English"),
    "force_ocr": (None, False),
    "paginate": (None, False),
    'output_format': (None, 'markdown'),
    "use_llm": (None, False),
    "strip_existing_ocr": (None, False),
    "disable_image_extraction": (None, False)
}

headers = {"X-Api-Key": "YOUR_API_KEY"}

response = requests.post(url, files=form_data, headers=headers)
data = response.json()

As you can see, everything is a form parameter. This is because we're uploading a file, so the request body has to be multipart/form-data.

Parameters: - file, the input file. - langs is an optional, comma-separated list of languages in the file (this is used if OCR is needed). The language names and codes are from here. - output_format - one of json, html, or markdown. - force_ocr will force OCR on every page (ignore the text in the PDF). This is slower, but can be useful for PDFs with known bad text. - paginate - adds delimiters to the output pages. See the API reference for details. - use_llm - setting this to True will use an LLM to enhance accuracy of forms, tables, inline math, and layout. It can be much more accurate, but carries a small hallucination risk. Setting use_llm to True will make responses slower. - strip_existing_ocr - setting to True will remove all existing OCR text from the file and redo OCR. This is useful if you know OCR text was added to the PDF by a low-quality OCR tool. - disable_image_extraction - setting to True will disable extraction of images. If use_llm is set to True, this will also turn images into text descriptions. - max_pages - from the start of the file, specifies the maximum number of pages to inference.

You can see a full list of parameters and descriptions in the API reference.

The request will return the following:

{'success': True, 'error': None, 'request_id': "PpK1oM-HB4RgrhsQhVb2uQ", 'request_check_url': 'https://www.datalab.to/api/v1/marker/PpK1oM-HB4RgrhsQhVb2uQ'}

You will then need to poll request_check_url, like this:

import time

max_polls = 300
check_url = data["request_check_url"]

for i in range(max_polls):
    time.sleep(2)
    response = requests.get(check_url, headers=headers) # Don't forget to send the auth headers
    data = response.json()

    if data["status"] == "complete":
        break

You can customize the max number of polls and the check interval to your liking. Eventually, the status field will be set to complete, and you will get an object that looks like this:

{
    "output_format": "markdown",
    "markdown": "...",
    "status": "complete",
    "success": True,
    "images": {...},
    "metadata": {...},
    "error": "",
    "page_count": 5
}

If success is False, you will get an error code along with the response.

All response data will be deleted from datalab servers an hour after the processing is complete, so make sure to get your results by then.
Response fields

    output_format is the requested output format, json, html, or markdown.
    markdown | json | html is the output from the file. It will be named according to the output_format. You can find more details on the json format here.
    status - indicates the status of the request (complete, or processing).
    success - indicates if the request completed successfully. True or False.
    images - dictionary of image filenames (keys) and base64 encoded images (values). Each value can be decoded with base64.b64decode(value). Then it can be saved to the filename (key).
    meta - metadata about the markdown conversion.
    error - if there was an error, this contains the error message.
    page_count - number of pages that were converted.

Supported file types

Marker supports the following extensions and mime types:

    PDF - pdf/application/pdf
    Spreadsheet - xls/application/vnd.ms-excel, xlsx/application/vnd.openxmlformats-officedocument.spreadsheetml.sheet, ods/application/vnd.oasis.opendocument.spreadsheet
    Word document - doc/application/msword, docx/application/vnd.openxmlformats-officedocument.wordprocessingml.document, odt/application/vnd.oasis.opendocument.text
    Powerpoint - ppt/application/vnd.ms-powerpoint, pptx/application/vnd.openxmlformats-officedocument.presentationml.presentation, odp/application/vnd.oasis.opendocument.presentation
    HTML - html/text/html
    Epub - epub/application/epub+zip
    Images - png/image/png, jpeg/image/jpeg, wepb/image/webp, gif/image/gif, tiff/image/tiff, jpg/image/jpg

You can automatically find the mimetype in python by installing filetype, then using filetype.guess(FILEPATH).mime.
Troubleshooting

If you get bad output, setting force_ocr to True is a good first step. A lot of PDFs have bad text inside. Marker attempts to auto-detect this and run OCR, but the auto-detection is not 100% accurate. Making sure langs is set properly is a good second step.
