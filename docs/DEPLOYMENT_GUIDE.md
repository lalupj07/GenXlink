# GenXLink - Free Tier Deployment Guide

**Deploy GenXLink for FREE using Railway/Fly.io + Supabase!**

---

## 🎯 **FREE TIER STACK**

### **Backend Hosting:**
- ✅ **Railway** (Free: $5/month credit) OR
- ✅ **Fly.io** (Free: 3 shared VMs, 160GB bandwidth)

### **Database:**
- ✅ **Supabase** (Free: 500MB database, 1GB file storage, 50MB bandwidth)

### **CI/CD:**
- ✅ **GitHub Actions** (Free: 2000 minutes/month)

### **API Docs:**
- ✅ **GitHub Pages** (Free: Unlimited static hosting)
- ✅ **SwaggerHub** (Free: 1 API, unlimited views)

### **Monitoring:**
- ✅ **UptimeRobot** (Free: 50 monitors, 5-min intervals)
- ✅ **Better Stack** (Free: 10K events/month)

---

## 🚀 **OPTION 1: DEPLOY TO RAILWAY**

### **Step 1: Setup Railway Account**
1. Go to https://railway.app
2. Sign up with GitHub (free)
3. Get $5/month credit (no credit card required!)

### **Step 2: Install Railway CLI**
```bash
npm install -g @railway/cli
railway login
```

### **Step 3: Deploy**
```bash
cd "c:/Users/lalup/OneDrive/Desktop/GenXis Innovations/GenXlink"

# Initialize Railway project
railway init

# Link to your project
railway link

# Deploy!
railway up
```

### **Step 4: Configure Environment**
```bash
# Set environment variables
railway variables set RUST_LOG=info
railway variables set DATABASE_URL=<your-supabase-url>
```

### **Step 5: Get Your URL**
```bash
railway domain
```

Your server will be at: `https://genxlink-server.up.railway.app`

---

## 🚀 **OPTION 2: DEPLOY TO FLY.IO**

### **Step 1: Setup Fly.io Account**
1. Go to https://fly.io
2. Sign up (free tier, no credit card!)
3. Get 3 shared VMs + 160GB bandwidth

### **Step 2: Install Fly CLI**
```bash
# Windows (PowerShell)
iwr https://fly.io/install.ps1 -useb | iex

# Verify installation
flyctl version
```

### **Step 3: Login**
```bash
flyctl auth login
```

### **Step 4: Deploy**
```bash
cd "c:/Users/lalup/OneDrive/Desktop/GenXis Innovations/GenXlink"

# Launch app (creates fly.toml if not exists)
flyctl launch

# Deploy
flyctl deploy
```

### **Step 5: Configure Secrets**
```bash
flyctl secrets set RUST_LOG=info
flyctl secrets set DATABASE_URL=<your-supabase-url>
```

### **Step 6: Get Your URL**
```bash
flyctl info
```

Your server will be at: `https://genxlink-server.fly.dev`

---

## 🗄️ **SETUP SUPABASE DATABASE**

### **Step 1: Create Supabase Project**
1. Go to https://supabase.com
2. Sign up (free)
3. Create new project
4. Choose region (Singapore for India)
5. Set database password

### **Step 2: Get Connection String**
1. Go to Project Settings → Database
2. Copy connection string (Pooler mode)
3. Replace `[YOUR-PASSWORD]` with your password

Example:
```
postgresql://postgres:[YOUR-PASSWORD]@db.abc123xyz.supabase.co:5432/postgres
```

### **Step 3: Create Tables**
```sql
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
```

### **Step 4: Enable Row Level Security (RLS)**
```sql
-- Enable RLS
ALTER TABLE devices ENABLE ROW LEVEL SECURITY;
ALTER TABLE sessions ENABLE ROW LEVEL SECURITY;

-- Allow public read (for now)
CREATE POLICY "Allow public read" ON devices FOR SELECT USING (true);
CREATE POLICY "Allow public read" ON sessions FOR SELECT USING (true);
```

---

## 📚 **SETUP API DOCUMENTATION**

### **Option A: GitHub Pages (Recommended)**

1. **Enable GitHub Pages:**
   - Go to your repo → Settings → Pages
   - Source: Deploy from branch
   - Branch: main, folder: /docs

2. **Access Docs:**
   - https://yourusername.github.io/GenXlink/api/

### **Option B: SwaggerHub**

1. **Create Account:**
   - Go to https://swagger.io/tools/swaggerhub/
   - Sign up (free tier)

2. **Import API:**
   - Create new API
   - Import `docs/api/swagger.yaml`
   - Publish

3. **Share:**
   - Get public URL
   - Share with team/users

---

## 🔄 **SETUP CI/CD WITH GITHUB ACTIONS**

### **Step 1: Add Secrets**
Go to GitHub repo → Settings → Secrets → Actions

Add these secrets:
- `RAILWAY_TOKEN` (from Railway dashboard)
- `FLY_API_TOKEN` (from `flyctl auth token`)
- `SUPABASE_URL`
- `SUPABASE_KEY`

### **Step 2: Push to GitHub**
```bash
git add .
git commit -m "Add deployment configuration"
git push origin main
```

### **Step 3: Watch Deployment**
- Go to Actions tab
- See your deployment running!
- Automatic deployment on every push

---

## 📊 **MONITORING & ANALYTICS**

### **UptimeRobot (Free)**
1. Go to https://uptimerobot.com
2. Add monitor:
   - Type: HTTP(S)
   - URL: https://your-server.railway.app/health
   - Interval: 5 minutes
3. Get alerts via email/Slack

### **Better Stack (Free)**
1. Go to https://betterstack.com
2. Add uptime monitor
3. Get detailed analytics

---

## 💰 **COST BREAKDOWN**

### **Free Tier Limits:**

**Railway:**
- $5/month credit (free)
- ~500 hours/month runtime
- Enough for 24/7 operation!

**Fly.io:**
- 3 shared VMs (free)
- 160GB bandwidth (free)
- Auto-scale to zero when idle

**Supabase:**
- 500MB database (free)
- 1GB file storage (free)
- 50MB bandwidth/day (free)
- Enough for 1000+ devices!

**GitHub Actions:**
- 2000 minutes/month (free)
- ~60 deployments/month

**Total Cost:** **$0/month** 🎉

### **When You Outgrow Free Tier:**

**Railway Pro:**
- $20/month
- Unlimited projects
- 8GB RAM per service

**Fly.io Paid:**
- Pay as you go
- ~$5-10/month for small app

**Supabase Pro:**
- $25/month
- 8GB database
- 100GB bandwidth

---

## 🎯 **DEPLOYMENT CHECKLIST**

### **Before Deployment:**
- [ ] All tests passing (100%)
- [ ] Environment variables configured
- [ ] Database schema created
- [ ] API documentation ready
- [ ] GitHub Actions configured

### **After Deployment:**
- [ ] Server health check passes
- [ ] WebSocket connection works
- [ ] Device registration works
- [ ] API docs accessible
- [ ] Monitoring setup
- [ ] SSL certificate active

---

## 🐛 **TROUBLESHOOTING**

### **Railway Deployment Fails:**
```bash
# Check logs
railway logs

# Restart service
railway restart

# Check variables
railway variables
```

### **Fly.io Deployment Fails:**
```bash
# Check logs
flyctl logs

# SSH into machine
flyctl ssh console

# Check status
flyctl status
```

### **Database Connection Issues:**
```bash
# Test connection
psql "postgresql://postgres:[PASSWORD]@db.abc.supabase.co:5432/postgres"

# Check Supabase dashboard
# Project → Database → Connection pooler
```

---

## 🚀 **QUICK DEPLOY COMMANDS**

### **Deploy to Railway:**
```bash
railway up
```

### **Deploy to Fly.io:**
```bash
flyctl deploy
```

### **Update Environment:**
```bash
# Railway
railway variables set KEY=value

# Fly.io
flyctl secrets set KEY=value
```

### **View Logs:**
```bash
# Railway
railway logs

# Fly.io
flyctl logs
```

---

## 🎊 **SUCCESS!**

Your GenXLink server is now:
- ✅ Deployed to cloud (free!)
- ✅ Connected to database (free!)
- ✅ Auto-deploying on push (free!)
- ✅ Monitored 24/7 (free!)
- ✅ API docs published (free!)

**Total cost: $0/month! 🎉**

---

**Next Steps:**
1. Deploy server
2. Test endpoints
3. Connect client
4. Share with users!

**🚀 YOU'RE LIVE! CONGRATULATIONS! 🎉**
