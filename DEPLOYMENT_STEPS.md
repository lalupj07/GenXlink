# 🚀 GenXLink - Deployment Steps

**Let's get your app LIVE!**

---

## ✅ **STEP 1: INITIALIZE GIT (If not already done)**

```bash
cd "c:/Users/lalup/OneDrive/Desktop/GenXis Innovations/GenXlink"

# Initialize git (if needed)
git init

# Add all files
git add .

# Commit
git commit -m "Initial commit: GenXLink v0.1.0 - Complete remote desktop solution"
```

---

## ✅ **STEP 2: CREATE GITHUB REPOSITORY**

### **Option A: Via GitHub Website**
1. Go to https://github.com/new
2. Repository name: `GenXlink`
3. Description: `Open-source remote desktop solution with 20+ features`
4. Choose: Public (for free GitHub Actions)
5. Click "Create repository"

### **Option B: Via GitHub CLI**
```bash
# Install GitHub CLI (if not installed)
# https://cli.github.com/

# Login
gh auth login

# Create repo
gh repo create GenXlink --public --description "Open-source remote desktop solution"
```

---

## ✅ **STEP 3: PUSH TO GITHUB**

```bash
# Add remote (replace YOUR_USERNAME)
git remote add origin https://github.com/YOUR_USERNAME/GenXlink.git

# Push to GitHub
git branch -M main
git push -u origin main
```

---

## ✅ **STEP 4: SETUP GITHUB ACTIONS SECRETS**

Go to: `https://github.com/YOUR_USERNAME/GenXlink/settings/secrets/actions`

Click "New repository secret" and add:

### **For Railway Deployment:**
- Name: `RAILWAY_TOKEN`
- Value: Get from https://railway.app/account/tokens

### **For Fly.io Deployment:**
- Name: `FLY_API_TOKEN`
- Value: Run `flyctl auth token`

### **For Supabase (later):**
- Name: `SUPABASE_URL`
- Value: Your Supabase project URL

- Name: `SUPABASE_KEY`
- Value: Your Supabase anon key

---

## ✅ **STEP 5: DEPLOY TO RAILWAY (EASIEST)**

### **Install Railway CLI:**
```bash
npm install -g @railway/cli
```

### **Login:**
```bash
railway login
```

### **Deploy:**
```bash
cd "c:/Users/lalup/OneDrive/Desktop/GenXis Innovations/GenXlink"

# Initialize Railway project
railway init

# Deploy!
railway up

# Get your URL
railway domain
```

**Your server will be live at:** `https://genxlink-server.up.railway.app`

---

## ✅ **STEP 6: SETUP SUPABASE DATABASE**

### **Create Project:**
1. Go to https://supabase.com
2. Click "New Project"
3. Name: `genxlink`
4. Database Password: (create strong password)
5. Region: Singapore (closest to India)
6. Click "Create new project"

### **Run SQL Schema:**
1. Go to SQL Editor
2. Paste this SQL:

```sql
-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Devices table
CREATE TABLE devices (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    device_id TEXT UNIQUE NOT NULL,
    device_name TEXT NOT NULL,
    connected_at TIMESTAMPTZ DEFAULT NOW(),
    last_seen TIMESTAMPTZ DEFAULT NOW(),
    is_online BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Sessions table
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id TEXT UNIQUE NOT NULL,
    from_device_id TEXT NOT NULL,
    to_device_id TEXT NOT NULL,
    started_at TIMESTAMPTZ DEFAULT NOW(),
    ended_at TIMESTAMPTZ,
    duration_seconds INTEGER,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_devices_device_id ON devices(device_id);
CREATE INDEX idx_devices_is_online ON devices(is_online);
CREATE INDEX idx_sessions_from_device ON sessions(from_device_id);
CREATE INDEX idx_sessions_to_device ON sessions(to_device_id);

-- Enable Row Level Security
ALTER TABLE devices ENABLE ROW LEVEL SECURITY;
ALTER TABLE sessions ENABLE ROW LEVEL SECURITY;

-- Allow public read (for now)
CREATE POLICY "Allow public read devices" ON devices FOR SELECT USING (true);
CREATE POLICY "Allow public read sessions" ON sessions FOR SELECT USING (true);
```

3. Click "Run"

### **Get Connection String:**
1. Go to Project Settings → Database
2. Copy "Connection string" (Pooler mode)
3. Replace `[YOUR-PASSWORD]` with your password

---

## ✅ **STEP 7: CONFIGURE ENVIRONMENT VARIABLES**

### **Railway:**
```bash
railway variables set DATABASE_URL="postgresql://..."
railway variables set RUST_LOG="info"
```

### **Fly.io:**
```bash
flyctl secrets set DATABASE_URL="postgresql://..."
flyctl secrets set RUST_LOG="info"
```

---

## ✅ **STEP 8: ENABLE GITHUB PAGES (API DOCS)**

1. Go to: `https://github.com/YOUR_USERNAME/GenXlink/settings/pages`
2. Source: "Deploy from a branch"
3. Branch: `main`
4. Folder: `/docs`
5. Click "Save"

**Your API docs will be at:** `https://YOUR_USERNAME.github.io/GenXlink/api/`

---

## ✅ **STEP 9: TEST YOUR DEPLOYMENT**

### **Test Server Health:**
```bash
curl https://genxlink-server.up.railway.app/health
```

**Expected Response:**
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "service": "genxlink-signaling-server"
}
```

### **Test WebSocket:**
Open browser console at your server URL and run:
```javascript
const ws = new WebSocket('wss://genxlink-server.up.railway.app/ws');
ws.onopen = () => console.log('Connected!');
ws.onmessage = (e) => console.log('Message:', e.data);
ws.send(JSON.stringify({
  type: 'register',
  device_id: 'test-123',
  device_name: 'Test Device'
}));
```

---

## ✅ **STEP 10: UPDATE CLIENT CONFIGURATION**

Edit `client/windows/src/config.rs`:

```rust
pub const SIGNALING_SERVER: &str = "wss://genxlink-server.up.railway.app/ws";
pub const API_SERVER: &str = "https://genxlink-server.up.railway.app";
```

Rebuild client:
```bash
cargo build --release --bin genxlink
```

---

## 🎯 **VERIFICATION CHECKLIST**

- [ ] Code pushed to GitHub
- [ ] GitHub Actions running
- [ ] Server deployed to Railway/Fly.io
- [ ] Server health check passes
- [ ] Supabase database created
- [ ] Database schema applied
- [ ] Environment variables set
- [ ] GitHub Pages enabled
- [ ] API docs accessible
- [ ] WebSocket connection works
- [ ] Client configured with server URL

---

## 📊 **YOUR LIVE URLS**

### **Server:**
- Railway: `https://genxlink-server.up.railway.app`
- Fly.io: `https://genxlink-server.fly.dev`

### **API Docs:**
- GitHub Pages: `https://YOUR_USERNAME.github.io/GenXlink/api/`

### **Database:**
- Supabase: `https://app.supabase.com/project/YOUR_PROJECT_ID`

### **Repository:**
- GitHub: `https://github.com/YOUR_USERNAME/GenXlink`

---

## 🐛 **TROUBLESHOOTING**

### **Railway deployment fails:**
```bash
railway logs
railway restart
```

### **GitHub Actions fails:**
- Check Actions tab for error logs
- Verify secrets are set correctly
- Check Cargo.toml syntax

### **Database connection fails:**
- Verify connection string
- Check Supabase project is running
- Test connection: `psql "postgresql://..."`

---

## 🎊 **SUCCESS INDICATORS**

✅ **Server is live** - Health check returns 200  
✅ **Database connected** - No connection errors  
✅ **WebSocket works** - Can register devices  
✅ **API docs live** - GitHub Pages accessible  
✅ **Auto-deploy works** - Push triggers deployment  

---

## 🚀 **NEXT STEPS AFTER DEPLOYMENT**

1. **Share with beta testers**
2. **Monitor server logs**
3. **Test client connections**
4. **Gather feedback**
5. **Iterate and improve**

---

## 💡 **QUICK COMMANDS REFERENCE**

```bash
# Deploy
railway up                    # Railway
flyctl deploy                 # Fly.io

# Logs
railway logs                  # Railway
flyctl logs                   # Fly.io

# Status
railway status                # Railway
flyctl status                 # Fly.io

# Environment
railway variables             # Railway
flyctl secrets list           # Fly.io

# Restart
railway restart               # Railway
flyctl apps restart           # Fly.io
```

---

**Version:** 0.1.0  
**Status:** 🚀 READY TO DEPLOY  
**Time to Live:** ~30 minutes  

**🎉 LET'S GO LIVE! YOU'VE GOT THIS! 🚀**
