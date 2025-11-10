import './globals.css';
import { Inter } from 'next/font/google';
import { getLocale } from 'next-intl/server';

const inter = Inter({ subsets: ['latin'] });

export const metadata = {
  title: 'AI Wallpaper Generator',
  description: 'Generate beautiful AI wallpapers with a simple prompt',
};

export default async function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  // 统一从服务端获取 locale，避免客户端水合时缺失 params 导致不一致
  const locale = (await getLocale())?.toLowerCase();
  const lang = locale && locale.startsWith('zh') ? 'zh-CN' : 'en';

  return (
    <html lang={lang} suppressHydrationWarning className={inter.className}>
      <body>
        {children}
      </body>
    </html>
  );
}
