import { cn } from "@/lib/utils"

interface SolanaLogoProps {
  className?: string
  size?: "sm" | "md" | "lg"
}

export function SolanaLogo({ className, size = "md" }: SolanaLogoProps) {
  const sizeClasses = {
    sm: "h-6 w-6",
    md: "h-8 w-8",
    lg: "h-10 w-10",
  }

  return (
    <div className={cn("relative flex items-center justify-center", sizeClasses[size], className)}>
      <div className="absolute inset-0 rounded-md bg-solana-gradient opacity-20 blur-sm"></div>
      <div className="relative flex flex-col gap-[3px]">
        <div className="h-[3px] w-full rounded-sm bg-solana-purple"></div>
        <div className="h-[3px] w-full rounded-sm bg-solana-blue"></div>
        <div className="h-[3px] w-full rounded-sm bg-solana-teal"></div>
      </div>
    </div>
  )
}
