# 🗺️ GenXLink Incremental Development Roadmap

**Strategy:** Implement one feature at a time, test thoroughly, then move to next  
**Timeline:** 2-3 months part-time  
**Status:** 🟢 Active Development

---

## 📅 **SPRINT SCHEDULE**

### **Sprint 1: Screen Capture** (Week 1-2) 🔄 **IN PROGRESS**
**Goal:** Capture screen and display in client

**Tasks:**
- [x] Create screen_capture.rs module
- [ ] Add Windows dependencies (windows-rs)
- [ ] Implement DXGI Desktop Duplication
- [ ] Add monitor selection
- [ ] Test frame capture
- [ ] Display captured frames in UI
- [ ] Add FPS control
- [ ] Performance optimization

**Deliverable:** App can capture and display local screen

---

### **Sprint 2: Video Encoding** (Week 3-4)
**Goal:** Compress captured frames for transmission

**Tasks:**
- [ ] Add video encoder (H.264)
- [ ] Implement hardware acceleration
- [ ] Add quality settings
- [ ] Test compression ratios
- [ ] Optimize encoding speed
- [ ] Add bitrate control

**Deliverable:** Captured frames are efficiently encoded

---

### **Sprint 3: WebRTC Streaming** (Week 5-6)
**Goal:** Send video to remote device

**Tasks:**
- [ ] Implement WebRTC peer connection
- [ ] Add ICE candidate exchange
- [ ] Set up data channels
- [ ] Test peer-to-peer connection
- [ ] Add connection recovery
- [ ] Implement reconnection logic

**Deliverable:** Video streams to remote device

---

### **Sprint 4: Input Injection** (Week 7-8)
**Goal:** Control remote computer

**Tasks:**
- [ ] Implement keyboard input (SendInput)
- [ ] Implement mouse input
- [ ] Add input validation
- [ ] Test input accuracy
- [ ] Add permission checks
- [ ] Implement input queue

**Deliverable:** Full remote control working

---

### **Sprint 5: Audio Streaming** (Week 9-10)
**Goal:** Transmit audio

**Tasks:**
- [ ] Implement WASAPI capture
- [ ] Add audio encoding (Opus)
- [ ] Implement audio playback
- [ ] Add volume control
- [ ] Test audio sync
- [ ] Optimize latency

**Deliverable:** Audio works with video

---

### **Sprint 6: File Transfer** (Week 11-12)
**Goal:** Share files between devices

**Tasks:**
- [ ] Implement file selection
- [ ] Add progress tracking
- [ ] Implement chunked transfer
- [ ] Add pause/resume
- [ ] Test large files
- [ ] Add transfer queue

**Deliverable:** Files can be transferred

---

## 🎯 **CURRENT SPRINT: Screen Capture**

### **What We're Building:**
A complete screen capture system using Windows DXGI that can:
1. Enumerate available monitors
2. Capture frames at 30 FPS
3. Convert to displayable format
4. Show in the client UI
5. Handle errors gracefully

### **Progress:**
- ✅ Created screen_capture.rs module
- ✅ Implemented DXGI initialization
- ✅ Added frame capture logic
- ✅ Added monitor enumeration
- ⏳ Need to add dependencies
- ⏳ Need to integrate with client
- ⏳ Need to add UI display

---

## 📊 **OVERALL PROGRESS**

```
Foundation:     ████████████████████ 100% ✅
Screen Capture: ████░░░░░░░░░░░░░░░░  20% 🔄
Video Encoding: ░░░░░░░░░░░░░░░░░░░░   0% ⏳
WebRTC:         ░░░░░░░░░░░░░░░░░░░░   0% ⏳
Input Control:  ░░░░░░░░░░░░░░░░░░░░   0% ⏳
Audio:          ░░░░░░░░░░░░░░░░░░░░   0% ⏳
File Transfer:  ░░░░░░░░░░░░░░░░░░░░   0% ⏳
```

**Total Progress:** 30% Complete

---

## 🔧 **NEXT ACTIONS**

### **This Week:**
1. Add Windows dependencies to Cargo.toml
2. Integrate screen_capture module
3. Add UI for screen preview
4. Test on multiple monitors
5. Fix any bugs

### **Testing Checklist:**
- [ ] Single monitor capture works
- [ ] Multi-monitor selection works
- [ ] FPS is stable at 30
- [ ] No memory leaks
- [ ] Error handling works
- [ ] UI updates smoothly

---

## 📝 **DEVELOPMENT NOTES**

### **Key Decisions:**
- Using DXGI for screen capture (native Windows, best performance)
- Target 30 FPS for smooth experience
- Hardware acceleration where possible
- Graceful degradation on older hardware

### **Known Challenges:**
- DXGI requires Windows 8+
- Need admin rights for some features
- Multi-monitor coordination
- Performance on low-end hardware

---

## 🎓 **LEARNING RESOURCES**

- DXGI Documentation: https://docs.microsoft.com/en-us/windows/win32/direct3ddxgi/
- WebRTC Rust: https://github.com/webrtc-rs/webrtc
- Windows Input: https://docs.microsoft.com/en-us/windows/win32/api/winuser/

---

**Last Updated:** November 23, 2025  
**Current Sprint:** Sprint 1 - Screen Capture  
**Next Review:** End of Week 2
