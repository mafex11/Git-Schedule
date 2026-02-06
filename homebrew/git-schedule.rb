# Homebrew formula for git-schedule
# Users install with: brew install mafex11/tap/git-schedule

class GitSchedule < Formula
  desc "Schedule git commits for later - like delayed send for your code"
  homepage "https://github.com/mafex11/Git-Schedule"
  version "0.1.0"
  license "MIT"

  on_macos do
    url "https://github.com/mafex11/Git-Schedule/releases/download/v#{version}/git-schedule-macos-aarch64.tar.gz"
    sha256 "6c1893d22a9d0bac1a111fe6d11eb487e51228691191eb5aef6b23b0cce16017"
  end

  on_linux do
    url "https://github.com/mafex11/Git-Schedule/releases/download/v#{version}/git-schedule-linux-x86_64.tar.gz"
    sha256 "024287e5d5af4457bc734973aef4667a0c639d4f1f51dbd1c8bb7e9e8b5d27b0"
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
