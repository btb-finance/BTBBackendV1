import '../styles/globals.css';
import type { AppProps } from 'next/app';
import { WalletConnectionProvider } from '../components/WalletConnectionProvider';

function MyApp({ Component, pageProps }: AppProps) {
  return (
    <WalletConnectionProvider>
      <Component {...pageProps} />
    </WalletConnectionProvider>
  );
}

export default MyApp;

