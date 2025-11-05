const http = require("http");

const server = http.createServer((req, res) => {
  res.setHeader("Content-Type", "text/html; charset=utf-8");

  if (req.url === "/" || req.url === "/dashboard") {
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
  } else if (req.url.startsWith("/policy-stores")) {
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
  } else if (req.url.startsWith("/settings")) {
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
  } else if (req.url.startsWith("/api-console")) {
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
  } else if (req.url.startsWith("/playground")) {
    res.writeHead(200);
    res.end(`<!DOCTYPE html>
<html>
<head>
    <title>Playground</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }
        .container { max-width: 1400px; margin: 0 auto; }
        .nav { background: #f0f0f0; padding: 10px; margin-bottom: 20px; }
        .nav a { margin-right: 20px; text-decoration: none; color: #333; }
        .nav a:hover { color: #007cba; }
        .tabs { display: flex; gap: 10px; margin-bottom: 20px; }
        .tab { padding: 10px 20px; background: white; border: 1px solid #ddd; cursor: pointer; border-radius: 4px; }
        .tab.active { background: #007cba; color: white; }
        .panel { background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        .section { margin-bottom: 20px; }
        input, textarea { padding: 8px; border: 1px solid #ddd; border-radius: 4px; margin-bottom: 10px; width: 100%; }
        button { background: #007cba; color: white; padding: 10px 20px; border: none; border-radius: 4px; cursor: pointer; }
        button:hover { background: #005a8a; }
        .test-suite { background: #f8f9fa; padding: 15px; margin: 10px 0; border-radius: 6px; }
        .badge { display: inline-block; padding: 4px 8px; border-radius: 12px; font-size: 0.85em; }
        .badge.success { background: #d4edda; color: #155724; }
        .badge.danger { background: #f8d7da; color: #721c24; }
        table { width: 100%; border-collapse: collapse; }
        th, td { padding: 10px; text-align: left; border-bottom: 1px solid #ddd; }
        .result-row { padding: 8px; margin: 5px 0; border-radius: 4px; }
        .result-allow { background: #d4edda; }
        .result-deny { background: #f8d7da; }
    </style>
</head>
<body>
    <div class="container">
        <div class="nav">
            <a href="/">Home</a>
            <a href="/dashboard">Dashboard</a>
            <a href="/policy-stores">Policy Stores</a>
            <a href="/playground">Playground</a>
            <a href="/search">Search</a>
            <a href="/settings">Settings</a>
            <a href="/api-console">API Console</a>
        </div>
        <h1>Playground - Batch Authorization Testing</h1>
        <div class="tabs">
            <button class="tab active">Single Test</button>
            <button class="tab">Batch Test</button>
        </div>
        <div class="panel">
            <h2>Predefined Test Suites</h2>
            <div class="test-suite">
                <h3>User Access Tests</h3>
                <p>Test common user access patterns</p>
                <button>User Access Tests (3 scenarios)</button>
            </div>
            <div class="test-suite">
                <h3>Role-Based Tests</h3>
                <p>Test role-based authorization</p>
                <button>Role-Based Tests (5 scenarios)</button>
            </div>
            <div class="test-suite">
                <h3>Custom Test Suite</h3>
                <p>Create your own test scenarios</p>
                <button>Configure Custom Test</button>
            </div>
            <div id="results" style="display:none;">
                <h3>Test Results</h3>
                <div class="result-row result-allow">
                    <strong>✓ Allow</strong> - User can access resource
                </div>
                <div class="result-row result-deny">
                    <strong>✗ Deny</strong> - User cannot access resource
                </div>
            </div>
        </div>
    </div>
    <script>
        document.querySelectorAll('.tab').forEach(tab => {
            tab.addEventListener('click', function() {
                document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
                this.classList.add('active');
            });
        });
        document.querySelectorAll('button').forEach(btn => {
            if (btn.textContent.includes('Tests')) {
                btn.addEventListener('click', function() {
                    document.getElementById('results').style.display = 'block';
                });
            }
        });
    </script>
</body>
</html>`);
  } else if (req.url.startsWith("/debug-mode")) {
    res.writeHead(200);
    res.end(`<!DOCTYPE html>
<html>
<head>
    <title>Debug Mode</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }
        .container { max-width: 1400px; margin: 0 auto; }
        .nav { background: #f0f0f0; padding: 10px; margin-bottom: 20px; }
        .nav a { margin-right: 20px; text-decoration: none; color: #333; }
        .debug-panel { background: #fff3cd; border: 2px solid #ffc107; padding: 20px; border-radius: 8px; margin-top: 20px; }
        .debug-step { background: white; padding: 10px; margin: 10px 0; border-left: 4px solid #007cba; }
        button { background: #007cba; color: white; padding: 10px 20px; border: none; border-radius: 4px; cursor: pointer; }
        button:hover { background: #005a8a; }
    </style>
</head>
<body>
    <div class="container">
        <div class="nav">
            <a href="/">Home</a>
            <a href="/dashboard">Dashboard</a>
            <a href="/policy-stores">Policy Stores</a>
            <a href="/playground">Playground</a>
            <a href="/debug-mode">Debug Mode</a>
        </div>
        <h1>Debug Mode</h1>
        <button onclick="toggleDebug()">Enable Debug Mode</button>
        <div id="debugPanel" class="debug-panel" style="display:none;">
            <h3>Debug Panel</h3>
            <div class="debug-step">
                <strong>Step 1:</strong> Evaluating policy conditions
                <div>Principal: user123</div>
                <div>Action: read</div>
                <div>Resource: document.pdf</div>
            </div>
            <div class="debug-step">
                <strong>Step 2:</strong> Checking permissions
                <div>Result: Permission granted</div>
            </div>
        </div>
    </div>
    <script>
        function toggleDebug() {
            const panel = document.getElementById('debugPanel');
            panel.style.display = panel.style.display === 'none' ? 'block' : 'none';
        }
    </script>
</body>
</html>`);
  } else if (req.url.startsWith("/scenarios")) {
    res.writeHead(200);
    res.end(`<!DOCTYPE html>
<html>
<head>
    <title>Scenarios</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .container { max-width: 1200px; margin: 0 auto; }
        .nav { background: #f0f0f0; padding: 10px; margin-bottom: 20px; }
        .nav a { margin-right: 20px; text-decoration: none; color: #333; }
        .scenario { background: white; padding: 15px; margin: 10px 0; border-radius: 6px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }
        input, textarea { padding: 8px; border: 1px solid #ddd; border-radius: 4px; margin-bottom: 10px; width: 100%; }
        button { background: #007cba; color: white; padding: 10px 20px; border: none; border-radius: 4px; cursor: pointer; }
        button:hover { background: #005a8a; }
    </style>
</head>
<body>
    <div class="container">
        <div class="nav">
            <a href="/">Home</a>
            <a href="/dashboard">Dashboard</a>
            <a href="/policy-stores">Policy Stores</a>
            <a href="/playground">Playground</a>
            <a href="/scenarios">Scenarios</a>
        </div>
        <h1>Scenarios Management</h1>
        <div class="scenario">
            <h3>Create New Scenario</h3>
            <input type="text" placeholder="Scenario Name" />
            <textarea placeholder="Description" rows="3"></textarea>
            <input type="text" placeholder="Principal" />
            <input type="text" placeholder="Action" />
            <input type="text" placeholder="Resource" />
            <button>Save Scenario</button>
        </div>
    </div>
</body>
</html>`);
  } else if (req.url.startsWith("/schemas")) {
    res.writeHead(200);
    res.end(`<!DOCTYPE html>
<html>
<head>
    <title>Schema Editor</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .container { max-width: 1200px; margin: 0 auto; }
        .nav { background: #f0f0f0; padding: 10px; margin-bottom: 20px; }
        .nav a { margin-right: 20px; text-decoration: none; color: #333; }
        .editor { background: white; padding: 15px; border-radius: 6px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }
        textarea { padding: 8px; border: 1px solid #ddd; border-radius: 4px; font-family: monospace; width: 100%; height: 400px; }
        button { background: #007cba; color: white; padding: 10px 20px; border: none; border-radius: 4px; cursor: pointer; margin-right: 10px; }
        button:hover { background: #005a8a; }
    </style>
</head>
<body>
    <div class="container">
        <div class="nav">
            <a href="/">Home</a>
            <a href="/dashboard">Dashboard</a>
            <a href="/policy-stores">Policy Stores</a>
            <a href="/schemas">Schema Editor</a>
        </div>
        <h1>Schema Editor</h1>
        <div class="editor">
            <textarea>{"name": "UserSchema", "attributes": ["id", "name", "role"]}</textarea>
            <div>
                <button>Save Schema</button>
                <button>Validate</button>
                <button>Duplicate</button>
                <button>Delete</button>
            </div>
        </div>
    </div>
</body>
</html>`);
  } else if (req.url.startsWith("/policies")) {
    res.writeHead(200);
    res.end(`<!DOCTYPE html>
<html>
<head>
    <title>Policy Editor</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .container { max-width: 1200px; margin: 0 auto; }
        .nav { background: #f0f0f0; padding: 10px; margin-bottom: 20px; }
        .nav a { margin-right: 20px; text-decoration: none; color: #333; }
        .wizard { background: white; padding: 15px; border-radius: 6px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }
        textarea { padding: 8px; border: 1px solid #ddd; border-radius: 4px; width: 100%; height: 300px; }
        input { padding: 8px; border: 1px solid #ddd; border-radius: 4px; margin-bottom: 10px; width: 100%; }
        button { background: #007cba; color: white; padding: 10px 20px; border: none; border-radius: 4px; cursor: pointer; margin-right: 10px; }
        button:hover { background: #005a8a; }
        .step { padding: 20px; border: 1px solid #ddd; border-radius: 4px; margin-bottom: 10px; }
    </style>
</head>
<body>
    <div class="container">
        <div class="nav">
            <a href="/">Home</a>
            <a href="/dashboard">Dashboard</a>
            <a href="/policy-stores">Policy Stores</a>
            <a href="/policies">Policy Editor</a>
        </div>
        <h1>Policy Editor</h1>
        <div class="wizard">
            <div class="step">
                <h3>Step 1: Basic Information</h3>
                <input type="text" placeholder="Policy Name" id="policyName" />
                <textarea placeholder="Policy Description" rows="3"></textarea>
            </div>
            <div class="step">
                <h3>Step 2: Policy Definition</h3>
                <textarea>permit(principal, action, resource);</textarea>
            </div>
            <div>
                <button>Next</button>
                <button>Previous</button>
                <button>Save Policy</button>
                <button>Validate</button>
            </div>
        </div>
    </div>
</body>
</html>`);
  } else if (req.url.startsWith("/snapshots")) {
    res.writeHead(200);
    res.end(`<!DOCTYPE html>
<html>
<head>
    <title>Snapshot Management</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .container { max-width: 1200px; margin: 0 auto; }
        .nav { background: #f0f0f0; padding: 10px; margin-bottom: 20px; }
        .nav a { margin-right: 20px; text-decoration: none; color: #333; }
        .snapshot { background: white; padding: 15px; margin: 10px 0; border-radius: 6px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); display: flex; justify-content: space-between; align-items: center; }
        .snapshot-info { flex: 1; }
        .snapshot-actions button { background: #007cba; color: white; padding: 8px 16px; border: none; border-radius: 4px; cursor: pointer; margin-left: 5px; }
        .snapshot-actions button:hover { background: #005a8a; }
        input { padding: 8px; border: 1px solid #ddd; border-radius: 4px; margin-bottom: 10px; width: 100%; }
        button { background: #007cba; color: white; padding: 10px 20px; border: none; border-radius: 4px; cursor: pointer; }
        button:hover { background: #005a8a; }
    </style>
</head>
<body>
    <div class="container">
        <div class="nav">
            <a href="/">Home</a>
            <a href="/dashboard">Dashboard</a>
            <a href="/policy-stores">Policy Stores</a>
            <a href="/snapshots">Snapshots</a>
        </div>
        <h1>Snapshot Management</h1>
        <div>
            <h3>Create Snapshot</h3>
            <input type="text" placeholder="Snapshot Description" id="snapshotDescription" />
            <button onclick="createSnapshot()">Create Snapshot</button>
        </div>
        <div id="snapshotList">
            <div class="snapshot">
                <div class="snapshot-info">
                    <h4>Snapshot 1</h4>
                    <p>Initial state - Oct 29, 2025</p>
                </div>
                <div class="snapshot-actions">
                    <button onclick="viewSnapshot()">View</button>
                    <button onclick="rollbackSnapshot()">Rollback</button>
                    <button onclick="deleteSnapshot()">Delete</button>
                </div>
            </div>
        </div>
    </div>
    <script>
        function createSnapshot() {
            alert('Snapshot created');
        }
        function viewSnapshot() {
            alert('Viewing snapshot');
        }
        function rollbackSnapshot() {
            if (confirm('Are you sure you want to rollback?')) {
                alert('Rollback complete');
            }
        }
        function deleteSnapshot() {
            if (confirm('Are you sure?')) {
                alert('Snapshot deleted');
            }
        }
    </script>
</body>
</html>`);
  } else if (req.url.startsWith("/templates")) {
    res.writeHead(200);
    res.end(`<!DOCTYPE html>
<html>
<head>
    <title>Templates System</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .container { max-width: 1200px; margin: 0 auto; }
        .nav { background: #f0f0f0; padding: 10px; margin-bottom: 20px; }
        .nav a { margin-right: 20px; text-decoration: none; color: #333; }
        .template { background: white; padding: 15px; margin: 10px 0; border-radius: 6px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }
        input { padding: 8px; border: 1px solid #ddd; border-radius: 4px; margin-bottom: 10px; width: 100%; }
        textarea { padding: 8px; border: 1px solid #ddd; border-radius: 4px; margin-bottom: 10px; width: 100%; }
        button { background: #007cba; color: white; padding: 10px 20px; border: none; border-radius: 4px; cursor: pointer; margin-right: 10px; }
        button:hover { background: #005a8a; }
    </style>
</head>
<body>
    <div class="container">
        <div class="nav">
            <a href="/">Home</a>
            <a href="/dashboard">Dashboard</a>
            <a href="/policy-stores">Policy Stores</a>
            <a href="/templates">Templates</a>
        </div>
        <h1>Templates System</h1>
        <div>
            <button onclick="showCreateModal()">Create Template</button>
            <input type="text" placeholder="Search templates..." style="width: 300px; float: right;" />
        </div>
        <div id="templateList">
            <div class="template">
                <h4>User Management Template</h4>
                <p>Common user authorization patterns</p>
                <button>Use Template</button>
            </div>
            <div class="template">
                <h4>File Access Template</h4>
                <p>File access control template</p>
                <button>Use Template</button>
            </div>
        </div>
        <div id="createModal" style="display:none; position: fixed; top: 50%; left: 50%; transform: translate(-50%, -50%); background: white; padding: 20px; border-radius: 8px; box-shadow: 0 4px 8px rgba(0,0,0,0.2);">
            <h3>Create Template</h3>
            <input type="text" placeholder="Template Name" />
            <textarea placeholder="Template Content" rows="5"></textarea>
            <div>
                <button onclick="saveTemplate()">Save</button>
                <button onclick="closeModal()">Cancel</button>
            </div>
        </div>
    </div>
    <script>
        function showCreateModal() {
            document.getElementById('createModal').style.display = 'block';
        }
        function closeModal() {
            document.getElementById('createModal').style.display = 'none';
        }
        function saveTemplate() {
            alert('Template saved');
            closeModal();
        }
    </script>
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
