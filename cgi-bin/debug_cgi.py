#!/usr/bin/env python3
"""
Debug CGI script to see exactly what the server passes to CGI
"""

import os
import sys
import cgitb

# Enable CGI error reporting
cgitb.enable()

def main():
    print("Content-Type: text/html; charset=utf-8")
    print()  # Empty line to end headers
    
    print("""<!DOCTYPE html>
<html>
<head>
    <title>CGI Debug Information</title>
    <style>
        body { font-family: monospace; margin: 20px; }
        .section { margin: 20px 0; padding: 10px; border: 1px solid #ccc; }
        .env-var { background: #f0f0f0; padding: 2px; }
    </style>
</head>
<body>
    <h1>🔍 CGI Debug Information</h1>""")
    
    # Show all environment variables
    print('<div class="section">')
    print('<h2>Environment Variables:</h2>')
    for key, value in sorted(os.environ.items()):
        print(f'<div class="env-var"><strong>{key}:</strong> {value}</div>')
    print('</div>')
    
    # Read stdin content once and store it (as binary data)
    stdin_data = None
    stdin_bytes = None
    print('<div class="section">')
    print('<h2>STDIN Content:</h2>')
    try:
        # Read as binary data to preserve multipart boundaries
        stdin_bytes = sys.stdin.buffer.read()
        if stdin_bytes:
            print(f'<pre>Length: {len(stdin_bytes)} bytes\n')
            # Convert to string for display, handling binary data safely
            try:
                stdin_data = stdin_bytes.decode('utf-8', errors='replace')
                display_data = stdin_data[:500]
                # Replace non-printable characters for safe HTML display
                safe_data = ''.join(c if ord(c) >= 32 and ord(c) < 127 else f'\\x{ord(c):02x}' for c in display_data)
                print(f'Content: {safe_data}</pre>')
                if len(stdin_data) > 500:
                    print('<p>... (truncated)</p>')
            except Exception as decode_error:
                print(f'<p>Binary data (cannot decode as UTF-8): {decode_error}</p>')
                # Show hex dump of first 100 bytes
                hex_data = ' '.join(f'{b:02x}' for b in stdin_bytes[:100])
                print(f'<pre>Hex dump (first 100 bytes): {hex_data}</pre>')
        else:
            print('<p>No STDIN data received</p>')
    except Exception as e:
        print(f'<p>Error reading STDIN: {e}</p>')
    print('</div>')
    
    # Show sys.argv
    print('<div class="section">')
    print('<h2>Command Line Arguments:</h2>')
    print(f'<pre>{sys.argv}</pre>')
    print('</div>')
    
    # Show current working directory
    print('<div class="section">')
    print('<h2>Working Directory:</h2>')
    print(f'<pre>{os.getcwd()}</pre>')
    print('</div>')
    
    # Try to parse form data manually
    print('<div class="section">')
    print('<h2>Manual Form Parsing Attempt:</h2>')
    try:
        import cgi
        import io

        # For POST requests, we need to create FieldStorage with proper parameters
        if os.environ.get('REQUEST_METHOD') == 'POST' and stdin_bytes:
            # Create a file-like object from the binary stdin data we read earlier
            stdin_file = io.BytesIO(stdin_bytes)

            # Create FieldStorage with explicit parameters
            form = cgi.FieldStorage(
                fp=stdin_file,
                environ=os.environ,
                keep_blank_values=True
            )
        else:
            form = cgi.FieldStorage()

        if form and len(form) > 0:
            print('<p>✅ FieldStorage created successfully</p>')
            print(f'<p>Form has {len(form)} field(s)</p>')
            print('<table border="1" style="border-collapse: collapse; margin: 10px 0;">')
            print('<tr><th>Field Name</th><th>Filename</th><th>Content Type</th><th>Value/Size</th></tr>')

            for key in form.keys():
                field = form[key]
                if hasattr(field, 'filename') and field.filename:
                    # This is a file upload
                    file_size = len(field.value) if hasattr(field, 'value') else 0
                    content_type = getattr(field, 'type', 'unknown')
                    print(f'<tr>')
                    print(f'<td><strong>{key}</strong></td>')
                    print(f'<td>{field.filename}</td>')
                    print(f'<td>{content_type}</td>')
                    print(f'<td>File size: {file_size} bytes</td>')
                    print(f'</tr>')

                    # Show first 200 characters of file content if it's text
                    if file_size > 0 and hasattr(field, 'value'):
                        try:
                            if isinstance(field.value, bytes):
                                preview = field.value.decode('utf-8', errors='replace')[:200]
                            else:
                                preview = str(field.value)[:200]
                            print(f'<tr><td colspan="4"><strong>File preview:</strong><br>')
                            print(f'<pre style="background: #f5f5f5; padding: 5px; max-height: 100px; overflow: auto;">{preview}{"..." if len(str(field.value)) > 200 else ""}</pre></td></tr>')
                        except Exception as e:
                            print(f'<tr><td colspan="4">Could not preview file: {e}</td></tr>')
                else:
                    # This is a regular form field
                    value = form.getvalue(key)
                    print(f'<tr>')
                    print(f'<td><strong>{key}</strong></td>')
                    print(f'<td>-</td>')
                    print(f'<td>text/plain</td>')
                    print(f'<td>{value}</td>')
                    print(f'</tr>')
            print('</table>')
        else:
            print('<p>❌ FieldStorage is empty or has no fields</p>')
            print(f'<p>Form object: {form}</p>')
            print(f'<p>Form length: {len(form) if form else "N/A"}</p>')

    except Exception as e:
        print(f'<p>❌ Error parsing form: {e}</p>')
        import traceback
        print(f'<pre style="background: #ffe6e6; padding: 10px; border: 1px solid #ff0000;">{traceback.format_exc()}</pre>')
    print('</div>')
    
    print('</body></html>')

if __name__ == "__main__":
    main()
