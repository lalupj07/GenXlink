# 🚀 NEXT STEPS - DEPLOYMENT GUIDE

**Your code is on GitHub! Now let's deploy it!**

Repository: https://github.com/lalupj07/GenXlink

---

## ✅ **STEP 1: ENABLE GITHUB ACTIONS (2 minutes)**

### **1. Go to Actions page:**
https://github.com/lalupj07/GenXlink/actions

### **2. Enable workflows:**
- You'll see: "Workflows aren't being run on this forked repository"
- Click: **"I understand my workflows, go ahead and enable them"**

### **3. Verify:**
- You should see the workflow: **"Deploy GenXLink"**
- Status: Ready to run on next push

---

## ✅ **STEP 2: SETUP GITHUB PAGES (3 minutes)**

### **1. Go to Pages settings:**
https://github.com/lalupj07/GenXlink/settings/pages

### **2. Configure:**
- **Source:** Deploy from a branch
- **Branch:** main
- **Folder:** /docs
- Click **"Save"**

### **3. Wait 2-3 minutes, then visit:**
https://lalupj07.github.io/GenXlink/

**Your API documentation will be live!**

---

## ✅ **STEP 3: DEPLOY TO RAILWAY (10 minutes)**

### **Option A: Using Railway CLI (Recommended)**

#### **1. Install Railway CLI:**
```powershell
npm install -g @railway/cli
```

#### **2. Login to Railway:**
```powershell
railway login
```
- Browser will open
- Login with GitHub
- Authorize Railway

#### **3. Initialize and Deploy:**
```powershell
cd "c:/Users/lalup/OneDrive/Desktop/GenXis Innovations/GenXlink"

# Initialize Railway project
railway init

# Link to your project (if you already created one)
# OR create new project
railway up

# Get your deployment URL
railway domain
```

#### **4. Set Environment Variables:**
```powershell
railway variables set RUST_LOG=info
railway variables set PORT=8080
```

### **Option B: Using Railway Dashboard**

#### **1. Go to Railway:**
https://railway.app/new

#### **2. Deploy from GitHub:**
- Click "Deploy from GitHub repo"
- Select: **lalupj07/GenXlink**
- Railway will auto-detect the `railway.toml` config

#### **3. Configure:**
- Service name: `genxlink-server`
- Root directory: `/server`
- Build command: Auto-detected from `railway.toml`
- Start command: Auto-detected from `railway.toml`

#### **4. Add Environment Variables:**
- Go to: Variables tab
- Add: `RUST_LOG=info`
- Add: `PORT=8080`

#### **5. Generate Domain:**
- Go to: Settings → Networking
- Click "Generate Domain"
- Your server will be at: `genxlink-server.up.railway.app`

---

## ✅ **STEP 4: SETUP SUPABASE DATABASE (10 minutes)**

### **1. Create Supabase Account:**
https://supabase.com/dashboard

### **2. Create New Project:**
- Project name: `genxlink`
- Database password: (create a strong password)
- Region: **Singapore** (closest to India)
- Pricing: **Free tier** (500MB)

### **3. Run Database Schema:**

Go to: SQL Editor in Supabase

Paste and run this SQL:

```sql
-- Create devices table
CREATE TABLE devices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255),
    platform VARCHAR(50),
    last_seen TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create sessions table
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id VARCHAR(255) UNIQUE NOT NULL,
    host_device_id UUID REFERENCES devices(id),
    client_device_id UUID REFERENCES devices(id),
    started_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    ended_at TIMESTAMP WITH TIME ZONE,
    status VARCHAR(50) DEFAULT 'active'
);

-- Create indexes
CREATE INDEX idx_devices_device_id ON devices(device_id);
CREATE INDEX idx_devices_last_seen ON devices(last_seen);
CREATE INDEX idx_sessions_status ON sessions(status);
```

### **4. Get Connection String:**
- Go to: Settings → Database
- Copy: **Connection string** (URI format)
- It looks like: `postgresql://postgres:[PASSWORD]@[HOST]:5432/postgres`

### **5. Add to Railway:**
```powershell
railway variables set DATABASE_URL="postgresql://postgres:[PASSWORD]@[HOST]:5432/postgres"
```

Or in Railway Dashboard:
- Go to: Variables
- Add: `DATABASE_URL` = your connection string

---

## ✅ **STEP 5: SETUP GITHUB SECRETS (5 minutes)**

### **1. Get Railway Token:**
```powershell
railway whoami
```
Or go to: https://railway.app/account/tokens
- Create new token
- Copy it

### **2. Add to GitHub:**
https://github.com/lalupj07/GenXlink/settings/secrets/actions

Click "New repository secret":
- **Name:** `RAILWAY_TOKEN`
- **Value:** (paste your token)
- Click "Add secret"

### **3. Optional - Fly.io Token:**
If you want to deploy to Fly.io too:
- Get token: `flyctl auth token`
- Add secret: `FLY_API_TOKEN`

---

## ✅ **STEP 6: TEST DEPLOYMENT (5 minutes)**

### **1. Check Railway Deployment:**
```powershell
railway status
```

### **2. Test Health Endpoint:**
```powershell
curl https://genxlink-server.up.railway.app/health
```

Expected response:
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime": 123
}
```

### **3. Test Devices Endpoint:**
```powershell
curl https://genxlink-server.up.railway.app/api/devices
```

Expected response:
```json
{
  "devices": []
}
```

### **4. Check Logs:**
```powershell
railway logs
```

---

## ✅ **STEP 7: UPDATE CLIENT CONFIG (2 minutes)**

Update your client to use the live server:

Edit: `client/windows/src/config.rs`

Change:
```rust
pub const SIGNALING_SERVER: &str = "ws://localhost:8080/ws";
```

To:
```rust
pub const SIGNALING_SERVER: &str = "wss://genxlink-server.up.railway.app/ws";
```

Commit and push:
```powershell
git add .
git commit -m "Update client to use production server"
git push
```

---

## 🎯 **DEPLOYMENT CHECKLIST:**

- [ ] GitHub Actions enabled
- [ ] GitHub Pages configured
- [ ] Railway CLI installed
- [ ] Railway project created
- [ ] Server deployed to Railway
- [ ] Domain generated
- [ ] Supabase project created
- [ ] Database schema created
- [ ] DATABASE_URL set in Railway
- [ ] RAILWAY_TOKEN added to GitHub
- [ ] Health endpoint tested
- [ ] Client config updated
- [ ] Changes pushed to GitHub

---

## 📊 **YOUR INFRASTRUCTURE:**

```
┌─────────────────────────────────────────┐
│           GITHUB (Code)                 │
│  https://github.com/lalupj07/GenXlink   │
└─────────────────┬───────────────────────┘
                  │
                  ├─► GitHub Actions (CI/CD)
                  │   - Build & Test
                  │   - Deploy to Railway
                  │
                  ├─► GitHub Pages (Docs)
                  │   https://lalupj07.github.io/GenXlink/
                  │
┌─────────────────▼───────────────────────┐
│        RAILWAY (Server)                 │
│  genxlink-server.up.railway.app         │
│  - Signaling Server                     │
│  - WebSocket Endpoint                   │
│  - REST API                             │
└─────────────────┬───────────────────────┘
                  │
                  │ DATABASE_URL
                  │
┌─────────────────▼───────────────────────┐
│       SUPABASE (Database)               │
│  - PostgreSQL                           │
│  - Devices Table                        │
│  - Sessions Table                       │
└─────────────────────────────────────────┘
```

---

## 💰 **COST BREAKDOWN:**

- **GitHub:** $0/month (Public repo + Actions)
- **Railway:** $0/month ($5 credit, enough for testing)
- **Supabase:** $0/month (500MB free tier)
- **GitHub Pages:** $0/month (Unlimited)

**Total: $0/month!** 🎉

---

## 🚀 **QUICK START (TL;DR):**

```powershell
# 1. Enable GitHub Actions
# Go to: https://github.com/lalupj07/GenXlink/actions

# 2. Install Railway CLI
npm install -g @railway/cli

# 3. Deploy to Railway
railway login
railway init
railway up
railway domain

# 4. Setup Supabase
# Go to: https://supabase.com
# Create project, run SQL schema

# 5. Set environment variables
railway variables set DATABASE_URL="your_supabase_url"

# 6. Test
curl https://genxlink-server.up.railway.app/health
```

---

## 🎊 **YOU'RE ALMOST LIVE!**

Follow these steps and your GenXLink server will be:
- ✅ Deployed on Railway
- ✅ Connected to Supabase
- ✅ Accessible worldwide
- ✅ Auto-deploying on push
- ✅ Monitored and logged

**Let's do this!** 🌍

---

**Start with STEP 1: Enable GitHub Actions**
**Go to: https://github.com/lalupj07/GenXlink/actions**
