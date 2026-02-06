# This formula should be placed in your homebrew-tap repository
# Users install with: brew install yourusername/tap/git-schedule

class GitSchedule < Formula
  desc "Schedule git commits for later - like delayed send for your code"
  homepage "https://github.com/mafex11/Git-Schedule"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/mafex11/Git-Schedule/releases/download/v#{version}/git-schedule-macos-aarch64.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256"
    else
      url "https://github.com/mafex11/Git-Schedule/releases/download/v#{version}/git-schedule-macos-x86_64.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256"
    end
  end

  on_linux do
    url "https://github.com/mafex11/Git-Schedule/releases/download/v#{version}/git-schedule-linux-x86_64.tar.gz"
    sha256 "REPLACE_WITH_ACTUAL_SHA256"
  end

  def install
    bin.install "git-schedule"
    bin.install "git-schedule-daemon"
  end

  def caveats
    <<~EOS
      git-schedule has been installed!

      Quick start:
        git add myfile.rs
        git-schedule "feat: my commit" --in 2h

      The daemon starts automatically when you schedule a commit.

      For more info: git-schedule --help
    EOS
  end

  test do
    assert_match "Schedule git commits", shell_output("#{bin}/git-schedule --help")
  end
end
