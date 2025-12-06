# GenXLink macOS Installer Formula
# This formula builds the complete macOS DMG installer for GenXLink

require "formula"

class Genxlink < Formula
  desc "GenXLink Remote Desktop - Secure, high-performance remote desktop solution"
  homepage "https://genxlink.com"
  url "https://github.com/genxlink/genxlink/archive/refs/tags/v0.2.0.tar.gz"
  sha256 "placeholder_sha256" # Will be updated during build
  license "MIT"
  head "https://github.com/genxlink/genxlink.git", branch: "main"

  depends_on "rust" => :build
  depends_on "cmake" => :build
  depends_on "pkg-config" => :build
  depends_on "openssl@3"
  depends_on "ffmpeg"
  depends_on "libvpx"
  depends_on "opus"
  depends_on "protobuf"

  # macOS specific dependencies
  on_macos do
    depends_on "coreutils"
    depends_on "gnu-sed"
  end

  def install
    # Set environment variables
    ENV["MACOSX_DEPLOYMENT_TARGET"] = "11.0"
    ENV["GENXLINK_VERSION"] = version
    ENV["GENXLINK_BUILD"] = "homebrew"

    # Build the application
    system "cargo", "build", "--release", "--bin", "genxlink"
    
    # Install main executable
    bin.install "target/release/genxlink" => "genxlink"
    
    # Install helper binaries if they exist
    if File.exist?("target/release/genxlink-service")
      bin.install "target/release/genxlink-service"
    end
    
    if File.exist?("target/release/genxlink-cli")
      bin.install "target/release/genxlink-cli"
    end

    # Install configuration files
    (etc/"genxlink").install "config/default.toml" => "default.toml"
    (etc/"genxlink").install "config/logging.toml" => "logging.toml"
    (etc/"genxlink").install "config/security.toml" => "security.toml"

    # Install documentation
    doc.install "README.md" => "README.md"
    doc.install "LICENSE" => "LICENSE"
    doc.install Dir["docs/*"]

    # Install application resources
    pkgshare.install "resources" => "resources"

    # Install macOS specific files
    if OS.mac?
      # Install LaunchAgent for auto-start
      (prefix/"Library/LaunchAgents").install "deployment/macos/com.genxlink.genxlink.plist"
      
      # Install application bundle (if exists)
      if File.exist?("target/release/bundle/macos/GenXLink.app")
        prefix.install "target/release/bundle/macos/GenXLink.app"
      end
    end

    # Create necessary directories
    (var/"log/genxlink").mkpath
    (var/"lib/genxlink").mkpath
    (var/"cache/genxlink").mkpath

    # Set proper permissions
    chmod 0755, bin/"genxlink"
    chmod 0755, var/"log/genxlink"
    chmod 0755, var/"lib/genxlink"
    chmod 0755, var/"cache/genxlink"
  end

  def post_install
    # Generate initial configuration
    config_dir = etc/"genxlink"
    if !config_dir.exist?
      config_dir.mkpath
    end

    # Create user configuration directory
    user_config = Pathname.new("#{ENV["HOME"]}/.config/genxlink")
    user_config.mkpath unless user_config.exist?

    # Set up log rotation
    logrotate_config = "#{var}/log/genxlink/*.log {
      daily
      missingok
      rotate 7
      compress
      delaycompress
      notifempty
      create 644 #{ENV["USER"]} staff
    }"
    
    (etc/"logrotate.d"/"genxlink").write logrotate_config if OS.linux?
  end

  def caveats
    <<~EOS
      GenXLink Remote Desktop has been installed!

      Configuration files are located in:
        #{etc}/genxlink/

      Logs will be written to:
        #{var}/log/genxlink/

      Data directory:
        #{var}/lib/genxlink/

      To start GenXLink:
        genxlink

      To enable auto-start (macOS):
        brew services start genxlink

      To enable auto-start (Linux):
        systemctl --user enable genxlink
        systemctl --user start genxlink

      For more information:
        https://docs.genxlink.com

      Note: GenXLink requires camera and microphone permissions for
      screen sharing and audio streaming functionality.
    EOS
  end

  service do
    run [opt_bin/"genxlink", "--service"]
    environment_variables PATH: std_service_path_env
    keep_alive true
    working_dir var/"lib/genxlink"
    log_path var/"log/genxlink/genxlink.log"
    error_log_path var/"log/genxlink/genxlink-error.log"
  end

  test do
    # Test basic functionality
    system bin/"genxlink", "--version"
    system bin/"genxlink", "--help"
    
    # Test configuration loading
    system bin/"genxlink", "--config", etc/"genxlink"/"default.toml", "--test-config"
    
    # Test service mode
    system bin/"genxlink", "--service", "--test"
  end

  # Update script for version upgrades
  def self.update_formula
    formula_path = Pathname.new(__FILE__)
    current_version = version.to_s
    
    # Fetch latest version from GitHub
    latest_tag = `git ls-remote --tags https://github.com/genxlink/genxlink.git | grep -E 'v[0-9]+\\.[0-9]+\\.[0-9]+$' | sort -V | tail -n1 | cut -d'/' -f3`.strip
    
    if latest_tag != "v#{current_version}"
      puts "New version available: #{latest_tag}"
      puts "Update formula with: brew bump-formula-pr genxlink --version=#{latest_tag.sub('v', '')}"
    else
      puts "Formula is up to date"
    end
  end
end
