const http = require('http');

const server = http.createServer((req, res) => {
  res.setHeader('Content-Type', 'text/html; charset=utf-8');
  
  if (req.url === '/' || req.url === '/dashboard') {
    res.writeHead(200);
    res.end(`<!DOCTYPE html>
<html>
<head>
    <title>Hodei Verified Permissions</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .container { max-width: 1200px; margin: 0 auto; }
        .nav { background: #f0f0f0; padding: 10px; margin-bottom: 20px; }
        .nav a { margin-right: 20px; text-decoration: none; color: #333; }
        .nav a:hover { color: #007cba; }
        .dashboard { background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        .metrics { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin: 20px 0; }
        .metric { background: #f8f9fa; padding: 15px; border-radius: 6px; text-align: center; }
        .metric h3 { margin: 0 0 10px 0; color: #333; }
        .metric .value { font-size: 2rem; font-weight: bold; color: #007cba; }
    </style>
</head>
<body>
    <div class="container">
        <div class="nav">
            <a href="/">Home</a>
            <a href="/dashboard">Dashboard</a>
            <a href="/policy-stores">Policy Stores</a>
            <a href="/search">Search</a>
            <a href="/settings">Settings</a>
            <a href="/api-console">API Console</a>
        </div>
        <div class="dashboard">
            <h1>Dashboard - Hodei Verified Permissions</h1>
            <p>Monitor your Hodei Verified Permissions system</p>
            
            <div class="metrics">
                <div class="metric">
                    <h3>Policy Stores</h3>
                    <div class="value">5</div>
                </div>
                <div class="metric">
                    <h3>Total Policies</h3>
                    <div class="value">42</div>
                </div>
                <div class="metric">
                    <h3>Active Templates</h3>
                    <div class="value">8</div>
                </div>
                <div class="metric">
                    <h3>Total Requests</h3>
                    <div class="value">1,247</div>
                </div>
            </div>
            
            <h2>Recent Activity</h2>
            <ul>
                <li>Authorization Request - 2 minutes ago</li>
                <li>Policy Created - 15 minutes ago</li>
                <li>Schema Updated - 1 hour ago</li>
                <li>Template Removed - 2 hours ago</li>
            </ul>
        </div>
    </div>
</body>
</html>`);
  } else if (req.url.startsWith('/policy-stores')) {
    res.writeHead(200);
    res.end(`<!DOCTYPE html>
<html>
<head>
    <title>Policy Stores</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .container { max-width: 1200px; margin: 0 auto; }
        .nav { background: #f0f0f0; padding: 10px; margin-bottom: 20px; }
        .nav a { margin-right: 20px; text-decoration: none; color: #333; }
        .store { background: white; padding: 15px; margin: 10px 0; border-radius: 6px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }
    </style>
</head>
<body>
    <div class="container">
        <div class="nav">
            <a href="/">Home</a>
            <a href="/dashboard">Dashboard</a>
            <a href="/policy-stores">Policy Stores</a>
            <a href="/search">Search</a>
            <a href="/settings">Settings</a>
            <a href="/api-console">API Console</a>
        </div>
        <h1>Policy Stores Management</h1>
        <div class="store">
            <h3>Production Policy Store</h3>
            <p>Main policy store for production environment</p>
            <span>Status: Active</span>
        </div>
        <div class="store">
            <h3>Development Policy Store</h3>
            <p>Testing and development policies</p>
            <span>Status: Active</span>
        </div>
    </div>
</body>
</html>`);
  } else if (req.url.startsWith('/settings')) {
    res.writeHead(200);
    res.end(`<!DOCTYPE html>
<html>
<head>
    <title>Settings</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .container { max-width: 1200px; margin: 0 auto; }
        .nav { background: #f0f0f0; padding: 10px; margin-bottom: 20px; }
        .nav a { margin-right: 20px; text-decoration: none; color: #333; }
        .tabs { display: flex; margin-bottom: 20px; }
        .tab { padding: 10px 20px; background: #f8f9fa; border: 1px solid #ddd; cursor: pointer; }
        .tab.active { background: #007cba; color: white; }
        .settings-panel { background: white; padding: 20px; border-radius: 6px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }
        .form-group { margin: 15px 0; }
        label { display: block; margin-bottom: 5px; font-weight: bold; }
        input, select { padding: 8px; border: 1px solid #ddd; border-radius: 4px; width: 200px; }
    </style>
</head>
<body>
    <div class="container">
        <div class="nav">
            <a href="/">Home</a>
            <a href="/dashboard">Dashboard</a>
            <a href="/policy-stores">Policy Stores</a>
            <a href="/search">Search</a>
            <a href="/settings">Settings</a>
            <a href="/api-console">API Console</a>
        </div>
        <h1>Configuration</h1>
        <p>Manage your application settings and preferences</p>
        
        <div class="tabs">
            <div class="tab active">User Preferences</div>
            <div class="tab">System Settings</div>
            <div class="tab">Feature Flags</div>
        </div>
        
        <div class="settings-panel">
            <h2>Theme & Appearance</h2>
            <div class="form-group">
                <label>Theme</label>
                <input type="radio" name="theme" value="light" checked> Light
                <input type="radio" name="theme" value="dark"> Dark
                <input type="radio" name="theme" value="system"> System
            </div>
            
            <div class="form-group">
                <label>Language</label>
                <select>
                    <option value="en">English</option>
                    <option value="es">Español</option>
                </select>
            </div>
            
            <h2>Notifications</h2>
            <div class="form-group">
                <label><input type="checkbox" checked> Email Notifications</label>
                <label><input type="checkbox" checked> Push Notifications</label>
                <label><input type="checkbox" checked> In-App Notifications</label>
            </div>
        </div>
    </div>
</body>
</html>`);
  } else if (req.url.startsWith('/api-console')) {
    res.writeHead(200);
    res.end(`<!DOCTYPE html>
<html>
<head>
    <title>API Console</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }
        .container { max-width: 1400px; margin: 0 auto; display: grid; grid-template-columns: 250px 1fr; gap: 20px; }
        .sidebar { background: white; padding: 20px; border-radius: 8px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }
        .main-content { background: white; padding: 20px; border-radius: 8px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }
        .request-builder { margin-bottom: 20px; }
        .tabs { display: flex; margin-bottom: 20px; }
        .tab { padding: 10px 20px; background: #f8f9fa; border: 1px solid #ddd; cursor: pointer; }
        .tab.active { background: #007cba; color: white; }
        input[type="url"], input[type="text"] { width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px; margin-bottom: 10px; }
        .btn { background: #007cba; color: white; padding: 8px 16px; border: none; border-radius: 4px; cursor: pointer; }
        .btn:hover { background: #005a8a; }
        .code-block { background: #f8f9fa; padding: 15px; border-radius: 4px; border-left: 4px solid #007cba; margin: 10px 0; }
    </style>
</head>
<body>
    <div class="container">
        <div class="sidebar">
            <h3>Request Collections</h3>
            <div style="margin-bottom: 15px;">
                <input type="text" placeholder="Search requests..." style="width: 100%; padding: 6px; border: 1px solid #ddd; border-radius: 4px;">
            </div>
            <ul style="list-style: none; padding: 0;">
                <li style="padding: 8px; background: #e3f2fd; border-radius: 4px; margin-bottom: 5px;">GET /policy-stores</li>
                <li style="padding: 8px; background: white; border-radius: 4px; margin-bottom: 5px;">POST /policies</li>
                <li style="padding: 8px; background: white; border-radius: 4px; margin-bottom: 5px;">GET /health</li>
            </ul>
        </div>
        
        <div class="main-content">
            <h1>API Console</h1>
            <p>Interactive API testing and documentation</p>
            
            <div class="tabs">
                <div class="tab active">Request</div>
                <div class="tab">Documentation</div>
                <div class="tab">History</div>
            </div>
            
            <div class="request-builder">
                <div style="display: grid; grid-template-columns: 100px 1fr auto; gap: 10px; margin-bottom: 15px;">
                    <select>
                        <option value="GET">GET</option>
                        <option value="POST">POST</option>
                        <option value="PUT">PUT</option>
                        <option value="DELETE">DELETE</option>
                    </select>
                    <input type="url" value="/policy-stores" placeholder="Enter request URL...">
                    <button class="btn">Send</button>
                </div>
                
                <div style="display: grid; grid-template-columns: 1fr 1fr auto; gap: 10px; margin-bottom: 15px;">
                    <input type="text" placeholder="Header name" value="Content-Type">
                    <input type="text" placeholder="Header value" value="application/json">
                    <button class="btn">Add Header</button>
                </div>
                
                <div style="display: grid; grid-template-columns: 1fr 1fr auto; gap: 10px; margin-bottom: 15px;">
                    <input type="text" placeholder="Parameter name" value="limit">
                    <input type="text" placeholder="Parameter value" value="10">
                    <button class="btn">Add Parameter</button>
                </div>
            </div>
            
            <div style="margin-bottom: 15px;">
                <strong>Response</strong>
                <div class="code-block">
                    <pre style="margin: 0; color: #2d3748;">{
  "policyStores": [
    {
      "id": "prod-store-1",
      "name": "Production Policy Store",
      "status": "active",
      "createdAt": "2025-10-29T10:00:00Z"
    }
  ],
  "total": 1,
  "page": 1,
  "size": 10
}

Status: 200 OK
Response Time: 45ms</pre>
                </div>
            </div>
            
            <div>
                <strong>Code Samples</strong>
                <div style="margin-top: 10px;">
                    <button class="btn" style="margin-right: 5px; background: #333;">curl</button>
                    <button class="btn" style="margin-right: 5px; background: #f7df1e; color: #333;">javascript</button>
                    <button class="btn" style="margin-right: 5px; background: #3776ab;">python</button>
                </div>
                <div class="code-block">
                    <pre style="margin: 0; color: #2d3748;">curl -X GET "http://localhost:3000/policy-stores" \
  -H "Content-Type: application/json"</pre>
                </div>
            </div>
        </div>
    </div>
</body>
</html>`);
  } else {
    res.writeHead(404);
    res.end(`<!DOCTYPE html>
<html>
<head><title>404 Not Found</title></head>
<body>
    <h1>404 - Page Not Found</h1>
    <p>The requested page could not be found.</p>
    <p><a href="/">Go to Home</a></p>
</body>
</html>`);
  }
});

const PORT = 3000;
server.listen(PORT, () => {
  console.log(`Test server running on http://localhost:${PORT}`);
});