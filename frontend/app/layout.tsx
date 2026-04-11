import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: "Targoo V2 | ESG Data Refinery",
  description: "Industrial ESG Data Normalization Engine",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" className="dark">
      <body className={`${inter.className} min-h-screen flex flex-col`}>
        <header className="border-b border-gray-800 bg-gray-900/50 backdrop-blur-md sticky top-0 z-50">
          <div className="max-w-5xl mx-auto px-6 h-16 flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="w-8 h-8 rounded bg-teal-500 flex items-center justify-center font-bold text-gray-950 shadow-[0_0_15px_rgba(14,207,207,0.3)]">T2</div>
              <span className="font-semibold tracking-wide text-lg">TARGOO <span className="text-gray-500 font-normal">Data Refinery</span></span>
            </div>
            <div className="text-xs text-gray-500 uppercase tracking-wider font-mono flex items-center gap-2">
              <span className="w-2 h-2 rounded-full bg-teal-500 animate-pulse"></span>
              Audit-Ready Engine
            </div>
          </div>
        </header>
        
        <main className="flex-1 max-w-5xl mx-auto w-full px-6 py-8">
          {children}
        </main>
      </body>
    </html>
  );
}
