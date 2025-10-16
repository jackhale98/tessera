import { ReactNode } from 'react';

interface CardProps {
  children: ReactNode;
  className?: string;
  title?: string;
  icon?: React.ReactNode;
}

export function Card({ children, className = '', title, icon }: CardProps) {
  return (
    <div className={`card ${className}`}>
      {(title || icon) && (
        <div className="flex items-center gap-2 px-5 pt-5 pb-4 border-b border-slate-100">
          {icon}
          {title && <h3 className="font-semibold text-slate-800 text-base">{title}</h3>}
        </div>
      )}
      <div className={title || icon ? 'p-5' : 'p-5'}>
        {children}
      </div>
    </div>
  );
}
