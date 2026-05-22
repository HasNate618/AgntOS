import { cn } from "@/lib/utils";

interface AgntLogoProps {
  className?: string;
  size?: number;
}

export default function AgntLogo({ className, size = 28 }: AgntLogoProps) {
  return (
    <img
      src="/agntos.svg"
      alt="AgntOS"
      width={size}
      height={size}
      className={cn("shrink-0", className)}
      style={{ height: size, width: size }}
    />
  );
}
