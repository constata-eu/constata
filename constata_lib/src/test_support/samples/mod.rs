pub fn multipart_email() -> String {
  concat!(
    "Subject: =?UTF-8?B?aG9saXMg8J+YhQ==?=\n",
    "Content-Type: multipart/alternative; boundary=foobar\n",
    "Date: Sun, 02 Oct 2016 07:06:22 -0700 (PDT)\n",
    "\n",
    "--foobar\n",
    "Content-Type: text/plain; charset=utf-8\n",
    "Content-Transfer-Encoding: quoted-printable\n",
    "Content-Disposition: attachment; filename=hello.txt\n",
    "\n",
    "This is the plaintext version, in utf-8. Proof by Euro: =E2=82=AC\n",
    "--foobar\n",
    "Content-Type: text/html\n",
    "Content-Transfer-Encoding: base64\n",
    "\n",
    "PGh0bWw+PGJvZHk+VGhpcyBpcyB0aGUgPGI+SFRNTDwvYj4gdmVyc2lvbiwgaW4g \n",
    "dXMtYXNjaWkuIFByb29mIGJ5IEV1cm86ICZldXJvOzwvYm9keT48L2h0bWw+Cg== \n",
    "--foobar\n",
    "Content-Type: application/zip\n",
    "Content-Transfer-Encoding: base64\n",
    "\n",
    "UEsDBBQAAAAAAIt2w1IAAAAAAAAAAAAAAAAEACAAYmFyL1VUDQAHh9C4YJzQuGCH \n",
    "0LhgdXgLAAEE6AMAAAToAwAAUEsDBBQACAAIAIt2w1IAAAAAAAAAAAQAAAALACAA \n",
    "YmFyL2Jhei50eHRVVA0AB4fQuGCH0Lhgh9C4YHV4CwABBOgDAAAE6AMAAEtKrOIC \n",
    "AFBLBwjhOXvMBgAAAAQAAABQSwMEFAAIAAgAe3bDUgAAAAAAAAAABAAAAAcAIABm \n",
    "b28udHh0VVQNAAdq0LhgatC4YGrQuGB1eAsAAQToAwAABOgDAABLy8/nAgBQSwcI \n",
    "qGUyfgYAAAAEAAAAUEsBAhQDFAAAAAAAi3bDUgAAAAAAAAAAAAAAAAQAIAAAAAAA \n",
    "AAAAAP1BAAAAAGJhci9VVA0AB4fQuGCc0Lhgh9C4YHV4CwABBOgDAAAE6AMAAFBL \n",
    "AQIUAxQACAAIAIt2w1LhOXvMBgAAAAQAAAALACAAAAAAAAAAAAC0gUIAAABiYXIv \n",
    "YmF6LnR4dFVUDQAHh9C4YIfQuGCH0LhgdXgLAAEE6AMAAAToAwAAUEsBAhQDFAAI \n",
    "AAgAe3bDUqhlMn4GAAAABAAAAAcAIAAAAAAAAAAAALSBoQAAAGZvby50eHRVVA0A \n",
    "B2rQuGBq0LhgatC4YHV4CwABBOgDAAAE6AMAAFBLBQYAAAAAAwADAAABAAD8AAAA \n",
    "AAA= \n",
    "--foobar--\n",
    "After the final boundary stuff gets ignored.\n"
  )
  .to_string()
}

pub fn multipart_email_base64() -> String {
  base64::encode(multipart_email().as_bytes())
}

pub fn json_docx_and_xlsx_email() -> String {
  concat!(
    "Subject: =?UTF-8?B?aG9saXMg8J+YhQ==?=\n",
    "Content-Type: multipart/alternative; boundary=foobar\n",
    "Date: Sun, 02 Oct 2016 07:06:22 -0700 (PDT)\n",
    "\n",
    "--foobar\n",
    "Content-Type: application/json; charset=utf-8\n",
    "Content-Transfer-Encoding: quoted-printable\n",
    "Content-Disposition: attachment; filename=json_for_testing.json\n",
    "\n",
    "{ \"created_at\": \"Thu May 19 14:20:59 +0000 2022\", \"id\": 1527293244064907266, \"id_str\": \"1527293244064907266\", \"full_text\": \"@rfiare @constataEu sella esto de tiempo tambien.\", \"truncated\": false, \"display_text_range\": [ 8, 49 ], \"entities\": { \"hashtags\": [], \"symbols\": [], \"user_mentions\": [ { \"screen_name\": \"rfiare\", \"name\": \"rfiare\", \"id\": 1430994392, \"id_str\": \"1430994392\", \"indices\": [ 0, 7 ] }, { \"screen_name\": \"constataEu\", \"name\": \"Constata.eu\", \"id\": 1333421127520169987, \"id_str\": \"1333421127520169987\", \"indices\": [ 8, 19 ] } ], \"urls\": [] }, \"metadata\": { \"iso_language_code\": \"es\", \"result_type\": \"recent\" }, \"source\": \"Twitter Web App\", \"in_reply_to_status_id\": 1527289116458622976, \"in_reply_to_status_id_str\": \"1527289116458622976\", \"in_reply_to_user_id\": 1430994392, \"in_reply_to_user_id_str\": \"1430994392\", \"in_reply_to_screen_name\": \"rfiare\" }\n",
    "--foobar\n",
    "Content-Type: application/vnd.openxmlformats-officedocument.wordprocessingml.document; charset=utf-8\n",
    "Content-Transfer-Encoding: quoted-printable\n",
    "Content-Disposition: attachment; filename=docx_for_testing.docx\n",
    "\n",
    "This is the docx version\n",
    "--foobar\n",
    "Content-Type: application/vnd.openxmlformats-officedocument.spreadsheetml.sheet; charset=utf-8\n",
    "Content-Transfer-Encoding: quoted-printable\n",
    "Content-Disposition: attachment; filename=xlsx_for_testing.xlsx\n",
    "\n",
    "This is the xlsx version\n",
    "--foobar\n",
    "Content-Type: application/vnd.openxmlformats-officedocument.presentationml.presentation; charset=utf-8\n",
    "Content-Transfer-Encoding: quoted-printable\n",
    "Content-Disposition: attachment; filename=pptx_for_testing.pptx\n",
    "\n",
    "This is the pptx version\n",
    "--foobar--\n",
    "After the final boundary stuff gets ignored.\n"
  )
  .to_string()
}
