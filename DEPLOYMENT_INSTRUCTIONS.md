# 🚀 GenXLink Cloud Deployment Instructions

## 📋 **DEPLOYMENT OPTIONS:**

### **Option A: Render.com (Recommended - Free Tier)**
1. **Push to GitHub**
   ```bash
   git remote add origin https://github.com/yourusername/genxlink.git
   git push -u origin main
   ```

2. **Deploy to Render**
   - Go to [render.com](https://render.com)
   - Sign up/login with GitHub
   - Click "New +" → "Web Service"
   - Connect your GitHub repository
   - Select `genxlink/server/signaling` as root directory
   - Use Dockerfile environment
   - Set plan to "Free"
   - Click "Deploy"

3. **Update Client URL**
   - Once deployed, get your URL: `your-app.onrender.com`
   - Update client: `ws://your-app.onrender.com/ws`

### **Option B: Railway (Alternative)**
1. **Upgrade Railway Account**
   - Go to [railway.app](https://railway.app)
   - Upgrade to Pro plan ($5/month)
   - Or use Hobby plan ($20/month)

2. **Deploy**
   ```bash
   railway login
   railway up
   ```

### **Option C: DigitalOcean ($4/month)**
1. **Create Droplet**
   - Go to [digitalocean.com](https://digitalocean.com)
   - Create Ubuntu 22.04 droplet (2GB RAM, $24/month or 1GB $6/month)
   - Or use App Platform for $4/month

2. **Deploy with Docker**
   ```bash
   # SSH into droplet
   ssh root@your-droplet-ip
   
   # Install Docker
   curl -fsSL https://get.docker.com -o get-docker.sh
   sh get-docker.sh
   
   # Clone and run
   git clone https://github.com/yourusername/genxlink.git
   cd genxlink/server/signaling
   docker build -t genxlink-signaling .
   docker run -p 8080:8080 genxlink-signaling
   ```

## 🌐 **POST-DEPLOYMENT:**

### **1. Test Cloud Server**
```bash
# Test health endpoint
curl https://your-app.onrender.com/health

# Expected: {"status":"healthy","service":"genxlink-signaling"}
```

### **2. Update Client Configuration**
```rust
// In client/windows/src/ui/streaming_panel.rs
signaling_server_url: "wss://your-app.onrender.com/ws".to_string(),
```

### **3. Test Cross-Network Discovery**
1. Run client on different networks
2. Test device discovery across internet
3. Verify WebRTC connections

## 🔧 **ENVIRONMENT VARIABLES:**
- `PORT`: Server port (default: 8080)
- `RUST_LOG`: Log level (info, debug, trace)

## 📊 **MONITORING:**
- Health check: `/health`
- WebSocket endpoint: `/ws`
- Logs available in platform dashboard

## 🎯 **NEXT STEPS:**
1. ✅ Deploy to cloud
2. ✅ Test cross-network connectivity
3. 🔄 Set up Supabase database
4. 🔄 Add authentication
5. 🔄 Performance optimization

## 📞 **QUICK COMMANDS:**
```bash
# Local test
cd server/signaling && cargo run

# Deploy to Render
git push origin main

# Test deployment
curl https://your-app.onrender.com/health
```

**🚀 Choose your deployment option and let's get GenXlink running in the cloud!**
