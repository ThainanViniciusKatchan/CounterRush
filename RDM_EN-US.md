[🇧🇷 Leia em Português](README.md)

| **COUNTER RUSH**                                                                        |
| --------------------------------------------------------------------------------------- |
| Play CS with higher performance, boost your FPS, eliminate stutter and frame drops.     |

> <span style="color:#e64340">Project still in early development stage; all modules are currently under testing and may change considerably until the release version.</span>

### Objective:

- Build a CS optimizer that is free and actually works, with no placebo commands or "tweaks".
- Simplify creating `.cfg` files or downloading pro players' settings.
- Better understand how tournament PCs are optimized and bring those techniques to casual players.
- Bring greater stability to the game I've played the most in my life.

#### What has already been done:

- [x] Launch the game directly through Steam from within the software.
- [x] Launch the game with updated CS2 parameters tailored to your hardware.
- [x] Disable CPU Core 0, as it is heavily used by background programs, avoiding thread contention between the game and open services.
- [x] Increase game process priority, keeping it prioritized by the CPU scheduler.
- [x] Disable GPU acceleration in background applications (Discord, Steam, and browsers), freeing up VRAM and GPU resources with optional rollback.
- [x] Purge free RAM by clearing standby pages (native clone of RAMMap's Empty Standby List).
- [x] DPI optimization and disabling DirectX fullscreen windowed optimizations (DPI Optimization).
- [x] Optimize RAM data reads by disabling memory compression.
- [x] Set system timer resolution to 0.5ms.
- [x] Optimize Multimedia Class Scheduler Service (MMCSS) tailored to hardware profiles, prioritizing network, CPU, GPU, and disk I/O for gaming.
- [x] Disable High Precision Event Timer (HPET) via BCD.
- [x] Disable Windows Game Mode, Xbox Game Bar shortcuts, Xbox Mode, and background captures (GameDVR).
- [x] Enable MSI Mode (Message Signaled-Based Interrupts) with High Priority for GPU, Network Adapters, and USB Host Controllers.

### What is currently in progress:

- [ ] Disable PCIe and USB Power Saving.
- [ ] Render Queue / Flip Queue Configuration.

### What is planned:

- Feature to create keybinds (in-game shortcuts) for all commands allowed in official and practice servers.
- Modern graphical interface (Slint) with modular switches and controls.
- Feature to configure a custom `autoexec.cfg` or download pro players' configs.
- Ability to browse and download Workshop maps directly through the software.

<p align="center">
  <a href="https://feitonobrasil.dev.br"><img src="https://selo.feitonobrasil.dev.br/en/branco-colorido/1x.svg" alt="Made in Brazil" height="56"></a>
</p>
