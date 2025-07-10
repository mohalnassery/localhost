#!/usr/bin/env python3
"""
Server statistics and resource monitoring CGI script
"""

import os
import sys
import cgi
import cgitb
from datetime import datetime
import time

# Enable CGI error reporting
cgitb.enable()

def format_bytes(bytes_val):
    """Format bytes in human readable format"""
    for unit in ['B', 'KB', 'MB', 'GB', 'TB']:
        if bytes_val < 1024.0:
            return f"{bytes_val:.1f} {unit}"
        bytes_val /= 1024.0
    return f"{bytes_val:.1f} PB"

def format_duration(seconds):
    """Format duration in human readable format"""
    if seconds < 60:
        return f"{seconds:.1f} seconds"
    elif seconds < 3600:
        return f"{seconds/60:.1f} minutes"
    elif seconds < 86400:
        return f"{seconds/3600:.1f} hours"
    else:
        return f"{seconds/86400:.1f} days"

def main():
    # Print HTTP headers
    print("Content-Type: text/html; charset=utf-8")
    print("Cache-Control: no-cache, no-store, must-revalidate")
    print("Pragma: no-cache")
    print("Expires: 0")
    print()  # Empty line to end headers
    
    # Get current time
    current_time = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    
    # Simulate server statistics (in a real implementation, these would come from the server)
    # For demonstration purposes, we'll show what the statistics would look like
    uptime_seconds = 3600  # 1 hour example
    total_requests = 1250
    total_bytes = 5242880  # 5MB example
    current_connections = 15
    max_connections = 1000
    peak_connections = 42
    error_count = 3
    
    # Calculate derived statistics
    requests_per_second = total_requests / uptime_seconds if uptime_seconds > 0 else 0
    bytes_per_second = total_bytes / uptime_seconds if uptime_seconds > 0 else 0
    error_rate = (error_count / total_requests * 100) if total_requests > 0 else 0
    connection_utilization = (current_connections / max_connections * 100) if max_connections > 0 else 0
    
    # Print HTML content
    print(f"""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Server Statistics</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            max-width: 1000px;
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
            margin-bottom: 30px;
        }}
        .stats-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 20px;
            margin: 20px 0;
        }}
        .stat-card {{
            background: #f8f9fa;
            padding: 20px;
            border-radius: 8px;
            border-left: 4px solid #007bff;
        }}
        .stat-card.warning {{
            border-left-color: #ffc107;
        }}
        .stat-card.danger {{
            border-left-color: #dc3545;
        }}
        .stat-card.success {{
            border-left-color: #28a745;
        }}
        .stat-title {{
            font-size: 0.9em;
            color: #666;
            margin-bottom: 5px;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }}
        .stat-value {{
            font-size: 2em;
            font-weight: bold;
            color: #333;
            margin-bottom: 5px;
        }}
        .stat-subtitle {{
            font-size: 0.8em;
            color: #888;
        }}
        .progress-bar {{
            width: 100%;
            height: 20px;
            background-color: #e9ecef;
            border-radius: 10px;
            overflow: hidden;
            margin: 10px 0;
        }}
        .progress-fill {{
            height: 100%;
            background-color: #007bff;
            transition: width 0.3s ease;
        }}
        .progress-fill.warning {{
            background-color: #ffc107;
        }}
        .progress-fill.danger {{
            background-color: #dc3545;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin: 20px 0;
        }}
        th, td {{
            text-align: left;
            padding: 12px;
            border-bottom: 1px solid #ddd;
        }}
        th {{
            background-color: #f8f9fa;
            font-weight: bold;
        }}
        .refresh-info {{
            text-align: center;
            margin: 20px 0;
            padding: 15px;
            background: #e3f2fd;
            border-radius: 5px;
            border-left: 4px solid #2196f3;
        }}
        .metric {{
            font-family: monospace;
            background: #f8f9fa;
            padding: 2px 6px;
            border-radius: 3px;
            font-size: 0.9em;
        }}
    </style>
    <script>
        // Auto-refresh every 30 seconds
        setTimeout(function() {{
            window.location.reload();
        }}, 30000);
    </script>
</head>
<body>
    <div class="container">
        <h1>📊 Server Statistics & Resource Monitor</h1>
        
        <div class="refresh-info">
            <strong>Last Updated:</strong> {current_time} | 
            <strong>Auto-refresh:</strong> Every 30 seconds
        </div>
        
        <div class="stats-grid">
            <div class="stat-card success">
                <div class="stat-title">Server Uptime</div>
                <div class="stat-value">{format_duration(uptime_seconds)}</div>
                <div class="stat-subtitle">Since last restart</div>
            </div>
            
            <div class="stat-card">
                <div class="stat-title">Total Requests</div>
                <div class="stat-value">{total_requests:,}</div>
                <div class="stat-subtitle">{requests_per_second:.2f} req/sec average</div>
            </div>
            
            <div class="stat-card">
                <div class="stat-title">Data Transferred</div>
                <div class="stat-value">{format_bytes(total_bytes)}</div>
                <div class="stat-subtitle">{format_bytes(bytes_per_second)}/sec average</div>
            </div>
            
            <div class="stat-card {'warning' if connection_utilization > 70 else 'success' if connection_utilization < 50 else ''}">
                <div class="stat-title">Active Connections</div>
                <div class="stat-value">{current_connections}</div>
                <div class="stat-subtitle">Peak: {peak_connections} | Limit: {max_connections}</div>
                <div class="progress-bar">
                    <div class="progress-fill {'warning' if connection_utilization > 70 else 'danger' if connection_utilization > 90 else ''}" 
                         style="width: {connection_utilization:.1f}%"></div>
                </div>
                <div class="stat-subtitle">{connection_utilization:.1f}% utilization</div>
            </div>
            
            <div class="stat-card {'danger' if error_rate > 5 else 'warning' if error_rate > 1 else 'success'}">
                <div class="stat-title">Error Rate</div>
                <div class="stat-value">{error_rate:.2f}%</div>
                <div class="stat-subtitle">{error_count} errors out of {total_requests} requests</div>
            </div>
            
            <div class="stat-card">
                <div class="stat-title">Server Software</div>
                <div class="stat-value" style="font-size: 1.2em;">Localhost HTTP</div>
                <div class="stat-subtitle">Version 0.1.0 | Rust Implementation</div>
            </div>
        </div>
        
        <h2>📈 Performance Metrics</h2>
        <table>
            <tr>
                <th>Metric</th>
                <th>Current Value</th>
                <th>Description</th>
            </tr>
            <tr>
                <td>Requests per Second</td>
                <td><span class="metric">{requests_per_second:.2f}</span></td>
                <td>Average request processing rate</td>
            </tr>
            <tr>
                <td>Bytes per Second</td>
                <td><span class="metric">{format_bytes(bytes_per_second)}</span></td>
                <td>Average data transfer rate</td>
            </tr>
            <tr>
                <td>Connection Utilization</td>
                <td><span class="metric">{connection_utilization:.1f}%</span></td>
                <td>Percentage of maximum connections in use</td>
            </tr>
            <tr>
                <td>Average Request Size</td>
                <td><span class="metric">{format_bytes(total_bytes / total_requests if total_requests > 0 else 0)}</span></td>
                <td>Average size of responses sent</td>
            </tr>
            <tr>
                <td>Error Rate</td>
                <td><span class="metric">{error_rate:.2f}%</span></td>
                <td>Percentage of requests resulting in errors</td>
            </tr>
        </table>
        
        <h2>🔧 Technical Information</h2>
        <table>
            <tr>
                <th>Property</th>
                <th>Value</th>
            </tr>
            <tr>
                <td>Server Name</td>
                <td>{os.environ.get('SERVER_NAME', 'localhost')}</td>
            </tr>
            <tr>
                <td>Server Port</td>
                <td>{os.environ.get('SERVER_PORT', '8888')}</td>
            </tr>
            <tr>
                <td>Server Software</td>
                <td>{os.environ.get('SERVER_SOFTWARE', 'localhost-http-server/0.1.0')}</td>
            </tr>
            <tr>
                <td>Gateway Interface</td>
                <td>{os.environ.get('GATEWAY_INTERFACE', 'CGI/1.1')}</td>
            </tr>
            <tr>
                <td>Request Method</td>
                <td>{os.environ.get('REQUEST_METHOD', 'GET')}</td>
            </tr>
            <tr>
                <td>User Agent</td>
                <td>{os.environ.get('HTTP_USER_AGENT', 'Unknown')}</td>
            </tr>
        </table>
        
        <div class="refresh-info">
            <h3>✅ Resource Management Features</h3>
            <p>✓ Connection timeout monitoring and cleanup</p>
            <p>✓ Request and response size tracking</p>
            <p>✓ Error rate monitoring and reporting</p>
            <p>✓ Connection limit enforcement</p>
            <p>✓ Performance metrics collection</p>
            <p>✓ Automatic resource cleanup</p>
        </div>
        
        <p style="text-align: center; margin-top: 30px;">
            <a href="/">← Return to Home</a> | 
            <a href="/cgi-bin/hello.py">CGI Test</a> |
            <a href="/cgi-bin/session_test.py">Session Test</a> |
            <a href="javascript:window.location.reload()">🔄 Refresh Stats</a>
        </p>
    </div>
</body>
</html>""")

if __name__ == "__main__":
    main()
