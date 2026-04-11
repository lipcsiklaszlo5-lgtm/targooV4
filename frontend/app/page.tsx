"use client";

import React, { useState, useRef, useEffect } from 'react';
import { UploadCloud, FileType, Play, Settings2, Trash2, Loader2, CheckCircle2, AlertCircle, Download } from 'lucide-react';
import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export default function Dashboard() {
  // Original states
  const [file, setFile] = useState<File | null>(null);
  const [year, setYear] = useState<string>("2024");
  const [language, setLanguage] = useState<string>("EN");
  const fileInputRef = useRef<HTMLInputElement>(null);

  // Phase 5: New states
  const [isProcessing, setIsProcessing] = useState(false);
  const [currentStep, setCurrentStep] = useState(0);
  const [quarantineCount, setQuarantineCount] = useState(0);
  const [isFinished, setIsFinished] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files[0]) {
      setFile(e.target.files[0]);
      setError(null);
    }
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    if (e.dataTransfer.files && e.dataTransfer.files[0]) {
      setFile(e.dataTransfer.files[0]);
      setError(null);
    }
  };

  const clearFile = (e: React.MouseEvent) => {
    e.stopPropagation();
    setFile(null);
    setIsFinished(false);
    setCurrentStep(0);
    if (fileInputRef.current) fileInputRef.current.value = "";
  };

  const getStepMessage = (step: number) => {
    switch (step) {
      case 1: return "Uploading source data...";
      case 2: return "Ingesting and normalizing...";
      case 3: return "Running AI classification...";
      case 4: return "Calculating GHG emissions...";
      case 5: return "Aggregating Scope 3 categories...";
      case 6: return "Finalizing Audit Package...";
      default: return "Initializing engine...";
    }
  };

  const handleGenerate = async () => {
    if (!file) return;

    setIsProcessing(true);
    setIsFinished(false);
    setError(null);
    setCurrentStep(1);

    try {
      // 1. Upload
      const formData = new FormData();
      formData.append('file', file);
      
      const uploadRes = await fetch('/api/upload', {
        method: 'POST',
        body: formData,
      });

      if (!uploadRes.ok) throw new Error("Upload failed");
      setCurrentStep(2);

      // 2. Run
      const runRes = await fetch('/api/run', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ 
          jurisdiction: year, 
          language: language 
        }),
      });

      if (!runRes.ok) throw new Error("Engine execution failed");

      // 3. Polling Status
      const pollInterval = setInterval(async () => {
        try {
          const statusRes = await fetch('/api/status');
          if (!statusRes.ok) return;
          
          const data = await statusRes.json();
          setCurrentStep(data.step);
          setQuarantineCount(data.quarantine_count);

          if (data.status === 'finished') {
            clearInterval(pollInterval);
            setIsProcessing(false);
            setIsFinished(true);
            setCurrentStep(6);
          }
        } catch (err) {
          console.error("Polling error:", err);
        }
      }, 2000);

    } catch (err: any) {
      setError(err.message);
      setIsProcessing(false);
    }
  };

  const formatSize = (bytes: number) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  return (
    <div className="space-y-8 animate-in fade-in duration-700">
      {/* Control Panel */}
      <section className="bg-gray-900/40 border border-gray-800 p-6 rounded-2xl flex flex-wrap gap-6 items-end">
        <div className="space-y-2">
          <label className="text-xs font-medium text-gray-500 uppercase tracking-widest flex items-center gap-2">
            <Settings2 className="w-3 h-3" /> Target Year
          </label>
          <select 
            disabled={isProcessing}
            value={year}
            onChange={(e) => setYear(e.target.value)}
            className="bg-gray-950 border border-gray-700 text-gray-200 text-sm rounded-lg focus:ring-teal-500 focus:border-teal-500 block w-40 p-2.5 outline-none transition-all disabled:opacity-50"
          >
            <option value="2023">Fiscal Year 2023</option>
            <option value="2024">Fiscal Year 2024</option>
          </select>
        </div>

        <div className="space-y-2">
          <label className="text-xs font-medium text-gray-500 uppercase tracking-widest flex items-center gap-2">
            <FileType className="w-3 h-3" /> Output Language
          </label>
          <select 
            disabled={isProcessing}
            value={language}
            onChange={(e) => setLanguage(e.target.value)}
            className="bg-gray-950 border border-gray-700 text-gray-200 text-sm rounded-lg focus:ring-teal-500 focus:border-teal-500 block w-40 p-2.5 outline-none transition-all disabled:opacity-50"
          >
            <option value="HU">Hungarian (HU)</option>
            <option value="EN">English (EN)</option>
            <option value="DE">German (DE)</option>
          </select>
        </div>
      </section>

      {/* Main Action Area */}
      {!isProcessing && !isFinished ? (
        <section 
          onDragOver={(e) => e.preventDefault()}
          onDrop={handleDrop}
          onClick={() => fileInputRef.current?.click()}
          className={cn(
            "relative group cursor-pointer overflow-hidden rounded-3xl border-2 border-dashed transition-all duration-300 min-h-[320px] flex flex-col items-center justify-center p-8",
            file 
              ? "border-teal-500/50 bg-teal-500/5 shadow-[0_0_30px_rgba(20,184,166,0.05)]" 
              : "border-gray-800 bg-gray-900/20 hover:border-gray-700 hover:bg-gray-900/40"
          )}
        >
          <input 
            type="file" 
            ref={fileInputRef}
            onChange={handleFileChange}
            className="hidden" 
            accept=".csv,.xlsx,.xls,.json"
          />

          {!file ? (
            <div className="flex flex-col items-center text-center space-y-4">
              <div className="p-5 bg-gray-800 rounded-full group-hover:scale-110 group-hover:bg-teal-500 transition-all duration-500">
                <UploadCloud className="w-10 h-10 text-gray-400 group-hover:text-gray-950" />
              </div>
              <div className="space-y-1">
                <h3 className="text-xl font-semibold text-gray-200">Ingest ESG Data</h3>
                <p className="text-gray-500 max-w-xs">Drag & drop your files here, or click to browse (CSV, Excel, JSON)</p>
              </div>
            </div>
          ) : (
            <div className="w-full max-w-md animate-in zoom-in-95 duration-300">
              <div className="bg-gray-950/80 backdrop-blur-sm border border-teal-500/30 p-5 rounded-2xl flex items-center gap-4 shadow-2xl">
                <div className="p-3 bg-teal-500 rounded-xl text-gray-950">
                  <FileType className="w-6 h-6" />
                </div>
                <div className="flex-1 min-w-0">
                  <p className="font-semibold text-gray-100 truncate">{file.name}</p>
                  <p className="text-xs text-gray-500 font-mono">{formatSize(file.size)}</p>
                </div>
                <button 
                  onClick={clearFile}
                  className="p-2 hover:bg-red-500/20 hover:text-red-400 rounded-lg transition-colors text-gray-600"
                >
                  <Trash2 className="w-5 h-5" />
                </button>
              </div>
            </div>
          )}
        </section>
      ) : isFinished ? (
        <section className="bg-gray-900/40 border border-teal-500/20 rounded-3xl p-12 text-center space-y-6 animate-in zoom-in-95 duration-500">
          <div className="flex justify-center">
            <div className="w-20 h-20 bg-teal-500 rounded-full flex items-center justify-center text-gray-950 shadow-[0_0_40px_rgba(20,184,166,0.3)]">
              <CheckCircle2 className="w-10 h-10" />
            </div>
          </div>
          <div className="space-y-2">
            <h3 className="text-2xl font-bold text-gray-100">Processing Complete</h3>
            <p className="text-gray-400">Your ESG Audit Package is ready for download.</p>
          </div>
          {quarantineCount > 0 && (
            <div className="inline-flex items-center gap-2 px-4 py-2 bg-amber-500/10 border border-amber-500/20 rounded-full text-amber-500 text-sm">
              <AlertCircle className="w-4 h-4" />
              {quarantineCount} items in quarantine for review
            </div>
          )}
          <div className="flex justify-center gap-4 pt-4">
            <a 
              href="/api/download" 
              className="bg-teal-500 text-gray-950 px-8 py-3 rounded-xl font-bold flex items-center gap-2 hover:bg-teal-400 transition-all"
            >
              <Download className="w-5 h-5" /> Download Result
            </a>
            <button 
              onClick={clearFile}
              className="bg-gray-800 text-gray-300 px-8 py-3 rounded-xl font-bold hover:bg-gray-700 transition-all"
            >
              Start New Session
            </button>
          </div>
        </section>
      ) : (
        <section className="bg-gray-900/40 border border-gray-800 rounded-3xl p-12 flex flex-col items-center justify-center space-y-8 animate-in fade-in duration-500">
          <div className="w-full max-w-md space-y-4">
            <div className="flex justify-between text-sm font-medium">
              <span className="text-teal-500">{getStepMessage(currentStep)}</span>
              <span className="text-gray-500">{Math.round((currentStep / 6) * 100)}%</span>
            </div>
            <div className="w-full h-3 bg-gray-800 rounded-full overflow-hidden">
              <div 
                className="h-full bg-teal-500 transition-all duration-700 ease-out shadow-[0_0_15px_rgba(20,184,166,0.5)]"
                style={{ width: `${(currentStep / 6) * 100}%` }}
              />
            </div>
          </div>
          <div className="flex items-center gap-3 text-gray-400 animate-pulse">
            <Loader2 className="w-5 h-5 animate-spin" />
            <span className="text-sm font-mono uppercase tracking-widest">Processing Data...</span>
          </div>
        </section>
      )}

      {/* Action Button */}
      {!isFinished && (
        <div className="flex justify-center">
          <button
            onClick={handleGenerate}
            disabled={!file || isProcessing}
            className={cn(
              "group relative px-10 py-4 rounded-2xl font-bold text-lg flex items-center gap-3 transition-all duration-300 transform active:scale-95",
              file && !isProcessing
                ? "bg-teal-500 text-gray-950 shadow-[0_0_20px_rgba(20,184,166,0.4)] hover:shadow-[0_0_30px_rgba(20,184,166,0.6)] hover:-translate-y-1" 
                : "bg-gray-800 text-gray-300 cursor-not-allowed grayscale opacity-50"
            )}
          >
            {isProcessing ? (
              <Loader2 className="w-5 h-5 animate-spin" />
            ) : (
              <Play className={cn("w-5 h-5 fill-current transition-transform", file && "group-hover:translate-x-1")} />
            )}
            {isProcessing ? "Processing..." : "Generate Audit Package"}
          </button>
        </div>
      )}

      {error && (
        <div className="max-w-md mx-auto p-4 bg-red-500/10 border border-red-500/20 rounded-xl flex items-start gap-3 text-red-400 text-sm animate-in slide-in-from-top-2">
          <AlertCircle className="w-5 h-5 shrink-0" />
          <p>{error}</p>
        </div>
      )}
    </div>
  );
}
