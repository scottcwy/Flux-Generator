# ⚡ Flux Generator

<div align="center">

[简体中文](./README.zh-CN.md) | English

</div>

✨ A clean and modern AI image generator built with Next.js, Tailwind CSS, and shadcn/ui. Generate stunning images with simple prompts.

## 🌟 Preview

<div align="center">
  <div>
    <img src="preview.png" alt="Flux Generator Interface" width="600px" />
    <br />
    <em>🎨 Clean and Modern Interface</em>
  </div>
</div>

## 🚀 Quick Start

```bash
# Clone the repository
git clone https://github.com/scottcwy/AI-Wallpaper.git

# Install dependencies
pnpm install

# Start development server
pnpm dev
```

## ✨ Features

- **🎨 AI Image Generation**: Create stunning images with simple text prompts
- **⚡ Modern UI**: Built with Tailwind CSS and shadcn/ui components
- **🌐 Internationalization**: Full i18n support with seamless language switching
- **🚀 One-Click Deploy**: Deploy to Vercel with zero configuration
- **📱 Responsive Design**: Perfect on desktop, tablet, and mobile devices
- **🔐 Authentication**: Secure user system with multiple providers

## ⚙️ Configuration

```bash
# Copy environment template
cp .env.example .env.local

# Required: Authentication secret for session JWT
# Generate a strong secret, for example:
# openssl rand -base64 32
AUTH_SECRET="your_auth_secret_key"

# Optional: OAuth providers
# Enable Google or GitHub by providing client credentials and toggles
AUTH_GOOGLE_ID="your_google_client_id"
AUTH_GOOGLE_SECRET="your_google_client_secret"
NEXT_PUBLIC_AUTH_GOOGLE_ENABLED="false"

AUTH_GITHUB_ID="your_github_client_id"
AUTH_GITHUB_SECRET="your_github_client_secret"
NEXT_PUBLIC_AUTH_GITHUB_ENABLED="false"

# Tip: In local development, if you forget to set AUTH_SECRET,
# the app falls back to a dev-only secret to avoid 500 errors.
# In production, always set AUTH_SECRET.
```

## 🎯 Usage

1. Enter your image prompt
2. Click generate and wait for AI magic
3. Download or share your creation

That's it! Simple and clean.

## 🚀 Deploy

[![Deploy with Vercel](https://vercel.com/button)](https://vercel.com/new/clone?repository-url=https%3A%2F%2Fgithub.com%2Fscottcwy%2FAI-Wallpaper)

One-click deployment to Vercel. No configuration needed.

## 🛠️ Tech Stack

- **Next.js 14** - React framework with App Router
- **Tailwind CSS** - Utility-first CSS framework
- **shadcn/ui** - Beautiful and accessible components
- **i18n** - Internationalization support
- **Vercel** - Deployment platform

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

---

**Simple. Clean. Powerful. 🎨**

