class AssemblyaiCli < Formula
  desc "CLI to transcribe audio/video files via AssemblyAI"
  homepage "https://github.com/diskd-ai/assemblyai-cli"
  url "https://github.com/diskd-ai/assemblyai-cli/archive/refs/tags/v0.1.4.tar.gz"
  sha256 "50a3674c9e8ddb82bcfecb3da45082e1323e7df3d803cb659881732ea0fc5e7a"

  depends_on "ffmpeg"
  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system "#{bin}/assemblyai-cli", "--version"
  end
end
