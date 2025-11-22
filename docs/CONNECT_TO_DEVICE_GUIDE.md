# How to Connect to a Remote Device

**GenXLink v0.1.0**  
**Feature:** Manual Device Connection

---

## 🎯 **NEW FEATURE ADDED!**

You can now **manually enter a device address** to connect to remote devices!

---

## 📱 **HOW TO CONNECT**

### **Method 1: From Devices Tab (Top Right)**

1. **Open GenXLink** application
2. **Go to "📱 Devices" tab**
3. **Click "➕ Connect to Device"** button (top right corner)
4. **Enter device information:**
   - **Device ID or IP:** Enter the remote device's IP address (e.g., `192.168.1.100`) or Device ID
   - **Device Name (optional):** Give it a friendly name (e.g., "Work Laptop")
5. **Click "Connect"** button
6. **Wait for connection** to establish

### **Method 2: When No Devices Found**

1. **Open GenXLink** application
2. **Go to "📱 Devices" tab**
3. If no devices are shown, you'll see:
   - "No devices found"
   - "Devices will appear here when they come online"
4. **Click "➕ Connect to Device Manually"** button
5. **Follow steps 4-6 from Method 1**

---

## 🔧 **CONNECTION DIALOG**

### **Input Screen:**
```
┌─────────────────────────────────────┐
│   Connect to Remote Device          │
│                                      │
│   Device ID or IP:                  │
│   [192.168.1.100 or device-id]      │
│                                      │
│   Device Name (optional):           │
│   [Work Laptop]                     │
│                                      │
│   [Connect]  [Cancel]               │
│                                      │
│   💡 Tip: You can enter an IP       │
│   address or a Device ID            │
└─────────────────────────────────────┘
```

### **Connecting Screen:**
```
┌─────────────────────────────────────┐
│   Connecting to Work Laptop         │
│                                      │
│   ⟳ (spinner)                       │
│                                      │
│   [████████░░░░] 60%                │
│                                      │
│   Finding best connection path...   │
│   Elapsed: 3s                       │
│                                      │
│   [Cancel]                          │
└─────────────────────────────────────┘
```

---

## 📝 **WHAT YOU CAN ENTER**

### **Device ID or IP Address:**

**Examples:**
- ✅ `192.168.1.100` - Local network IP
- ✅ `10.0.0.50` - Private network IP
- ✅ `device-abc-123-xyz` - Device ID
- ✅ `my-device-id-here` - Custom Device ID

### **Device Name (Optional):**

**Examples:**
- ✅ `Work Laptop` - Friendly name
- ✅ `Home PC` - Easy to remember
- ✅ `Server 1` - Descriptive name
- ✅ Leave empty - Will use IP/ID as name

---

## 🔄 **CONNECTION PROCESS**

### **Steps:**

1. **Initializing** - Setting up connection
2. **Connecting to signaling server** - Establishing communication
3. **Exchanging connection details** - Sharing connection info
4. **Finding best connection path** - Optimizing route
5. **Establishing peer connection** - Creating direct link
6. **Connected successfully!** - Ready to use

### **Progress Indicators:**
- **Progress Bar:** Shows completion percentage
- **Status Message:** Current step description
- **Elapsed Time:** How long the connection has been running
- **Spinner:** Visual indication of activity

---

## ❌ **IF CONNECTION FAILS**

### **Error Screen:**
```
┌─────────────────────────────────────┐
│   ❌                                 │
│                                      │
│   Failed to connect to Work Laptop  │
│                                      │
│   Connection timeout: Unable to     │
│   reach device                      │
│                                      │
│   [Retry]  [Close]                  │
└─────────────────────────────────────┘
```

### **What to Do:**
1. **Check IP address** - Make sure it's correct
2. **Check network** - Ensure both devices are connected
3. **Check firewall** - May be blocking connection
4. **Try again** - Click "Retry" button
5. **Close dialog** - Click "Close" to cancel

---

## 💡 **TIPS**

### **For Best Results:**

1. **Use IP Address** - More reliable than Device ID
2. **Same Network** - Both devices should be on same network
3. **Check Firewall** - Allow GenXLink through firewall
4. **Port Forwarding** - May be needed for external connections
5. **Device Online** - Make sure remote device is running GenXLink

### **Common Issues:**

**Problem:** "Connection timeout"
- **Solution:** Check if remote device is online and reachable

**Problem:** "Connection refused"
- **Solution:** Ensure GenXLink is running on remote device

**Problem:** "Network unreachable"
- **Solution:** Check network connectivity and firewall settings

---

## 🎯 **QUICK REFERENCE**

### **Button Locations:**

**Top Right (Devices Tab):**
```
┌──────────────────────────────────────────┐
│ Available Devices    [➕ Connect to Device]│
│                                          │
│ [Device List Here]                       │
└──────────────────────────────────────────┘
```

**Center (No Devices):**
```
┌──────────────────────────────────────────┐
│ Available Devices                        │
│                                          │
│         No devices found                 │
│   Devices will appear here when online   │
│                                          │
│   [➕ Connect to Device Manually]        │
└──────────────────────────────────────────┘
```

---

## 🚀 **READY TO CONNECT!**

**You now have two ways to connect:**
1. ✅ **Automatic** - Devices appear automatically when online
2. ✅ **Manual** - Enter device address manually

**Start connecting to your remote devices now!** 🎉

---

**Version:** 0.1.0  
**Feature:** Manual Device Connection  
**Status:** ✅ Available Now  
