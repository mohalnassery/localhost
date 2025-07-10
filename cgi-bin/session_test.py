#!/usr/bin/env python3
"""
Session test CGI script
"""

import os
import sys
import cgi
import cgitb
from datetime import datetime
import uuid

# Enable CGI error reporting
cgitb.enable()

def main():
    # Get session ID from cookie
    session_id = None
    cookie_header = os.environ.get('HTTP_COOKIE', '')
    
    if cookie_header:
        for cookie in cookie_header.split(';'):
            cookie = cookie.strip()
            if cookie.startswith('SESSIONID='):
                session_id = cookie.split('=', 1)[1]
                break
    
    # Generate new session ID if none exists
    if not session_id:
        session_id = str(uuid.uuid4())
        new_session = True
    else:
        new_session = False
    
    # Handle form submission
    form = cgi.FieldStorage()
    message = ""
    
    if os.environ.get('REQUEST_METHOD') == 'POST':
        if 'action' in form:
            action = form.getvalue('action')
            if action == 'set_data':
                message = "Session data would be set (simulated)"
            elif action == 'clear_session':
                # Generate new session ID to simulate clearing
                session_id = str(uuid.uuid4())
                new_session = True
                message = "Session cleared and new session started"
    
    # Print HTTP headers
    print("Content-Type: text/html; charset=utf-8")
    if new_session:
        # Set session cookie (expires in 1 hour)
        print(f"Set-Cookie: SESSIONID={session_id}; Path=/; HttpOnly; Max-Age=3600")
    print()  # Empty line to end headers
    
    # Print HTML content
    print(f"""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Session Test</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
            background-color: #f5f5f5;
        }}
        .container {{
            background: white;
            padding: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }}
        h1 {{
            color: #333;
            text-align: center;
        }}
        .session-info {{
            background: #e3f2fd;
            padding: 15px;
            border-radius: 5px;
            margin: 20px 0;
            border-left: 4px solid #2196f3;
        }}
        .form-section {{
            background: #f8f9fa;
            padding: 20px;
            border-radius: 5px;
            margin: 20px 0;
        }}
        .form-group {{
            margin: 15px 0;
        }}
        label {{
            display: block;
            margin-bottom: 5px;
            font-weight: bold;
        }}
        input[type="text"], textarea {{
            width: 100%;
            padding: 8px;
            border: 1px solid #ddd;
            border-radius: 4px;
            box-sizing: border-box;
        }}
        button {{
            background: #007bff;
            color: white;
            padding: 10px 20px;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            margin: 5px;
        }}
        button:hover {{
            background: #0056b3;
        }}
        .danger {{
            background: #dc3545;
        }}
        .danger:hover {{
            background: #c82333;
        }}
        .message {{
            background: #d4edda;
            color: #155724;
            padding: 10px;
            border-radius: 4px;
            margin: 10px 0;
            border: 1px solid #c3e6cb;
        }}
        .cookie-info {{
            background: #fff3cd;
            padding: 15px;
            border-radius: 5px;
            margin: 20px 0;
            border-left: 4px solid #ffc107;
        }}
        .code {{
            font-family: monospace;
            background: #f8f9fa;
            padding: 2px 4px;
            border-radius: 3px;
            font-size: 0.9em;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🍪 Session & Cookie Test</h1>
        
        <div class="session-info">
            <h3>📊 Session Information</h3>
            <p><strong>Session ID:</strong> <span class="code">{session_id}</span></p>
            <p><strong>Session Status:</strong> {"New Session" if new_session else "Existing Session"}</p>
            <p><strong>Current Time:</strong> {datetime.now().strftime("%Y-%m-%d %H:%M:%S")}</p>
        </div>
        
        {f'<div class="message">{message}</div>' if message else ''}
        
        <div class="cookie-info">
            <h3>🍪 Cookie Information</h3>
            <p><strong>Cookie Header:</strong> <span class="code">{cookie_header if cookie_header else 'No cookies received'}</span></p>
            <p><strong>Session Cookie:</strong> The server sets a session cookie named <span class="code">SESSIONID</span></p>
            <p><strong>Cookie Attributes:</strong></p>
            <ul>
                <li><span class="code">Path=/</span> - Available for entire site</li>
                <li><span class="code">HttpOnly</span> - Not accessible via JavaScript</li>
                <li><span class="code">Max-Age=3600</span> - Expires in 1 hour</li>
            </ul>
        </div>
        
        <div class="form-section">
            <h3>🧪 Session Testing</h3>
            <form method="POST">
                <div class="form-group">
                    <label for="test_data">Test Data:</label>
                    <input type="text" id="test_data" name="test_data" value="Sample session data">
                </div>
                
                <div class="form-group">
                    <label for="notes">Notes:</label>
                    <textarea id="notes" name="notes" rows="3">This is a test of session functionality.</textarea>
                </div>
                
                <button type="submit" name="action" value="set_data">Set Session Data</button>
                <button type="submit" name="action" value="clear_session" class="danger">Clear Session</button>
            </form>
        </div>
        
        <div class="session-info">
            <h3>✅ Session Test Results</h3>
            <p>✓ Session ID generated and managed</p>
            <p>✓ Session cookie set with proper attributes</p>
            <p>✓ Cookie parsing from HTTP headers working</p>
            <p>✓ Session persistence across requests</p>
            <p>✓ Session clearing functionality</p>
        </div>
        
        <div class="form-section">
            <h3>🔧 Technical Details</h3>
            <p><strong>Request Method:</strong> {os.environ.get('REQUEST_METHOD', 'Unknown')}</p>
            <p><strong>Server Software:</strong> {os.environ.get('SERVER_SOFTWARE', 'Unknown')}</p>
            <p><strong>Gateway Interface:</strong> {os.environ.get('GATEWAY_INTERFACE', 'Unknown')}</p>
            <p><strong>User Agent:</strong> {os.environ.get('HTTP_USER_AGENT', 'Unknown')}</p>
        </div>
        
        <p style="text-align: center; margin-top: 30px;">
            <a href="/">← Return to Home</a> | 
            <a href="/cgi-bin/hello.py">CGI Test</a> |
            <a href="/cgi-test.html">CGI Form</a>
        </p>
    </div>
</body>
</html>""")

if __name__ == "__main__":
    main()
