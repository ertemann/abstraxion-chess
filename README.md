# JSON Store Frontend

A Next.js frontend for interacting with the JSON Store smart contract on the XION blockchain.

## Prerequisites

- Node.js (v14 or higher)
- npm (v6 or higher)

## Quick Installation

1. Clone this repository:
   ```bash
   git clone <repository-url>
   cd <repository-name>
   ```

2. Make the setup script executable:
   ```bash
   chmod +x setup.sh
   ```

3. Run the setup script:
   ```bash
   ./setup.sh
   ```

   The script will:
   - Install dependencies
   - Prompt for your configuration values
   - Create the necessary environment files
   - Build the application

4. Start the application:
   ```bash
   # Development mode
   npm run dev
   
   # OR Production mode
   npm start
   ```

## Manual Configuration

If you prefer to configure the application manually:

1. Install dependencies:
   ```bash
   npm install
   ```

2. Create a `.env.local` file with the following variables:
   ```
   NEXT_PUBLIC_TREASURY_ADDRESS=your_treasury_address
   NEXT_PUBLIC_CONTRACT_ADDRESS=your_contract_address
   NEXT_PUBLIC_RPC_URL=your_rpc_url
   NEXT_PUBLIC_REST_URL=your_rest_url
   ```

3. Build and start the application:
   ```bash
   npm run build
   npm start
   ```

## Features

- Connect wallet using Abstraxion
- View and submit JSON data to the smart contract
- Query stored JSON data by user address
- View all users who have stored data
- View the complete map of stored data

## Environment Variables

- `NEXT_PUBLIC_TREASURY_ADDRESS`: The treasury address for the XION blockchain
- `NEXT_PUBLIC_CONTRACT_ADDRESS`: The address of the deployed JSON Store contract
- `NEXT_PUBLIC_RPC_URL`: The RPC URL for the XION blockchain (default: https://rpc.xion-testnet-2.burnt.com:443)
- `NEXT_PUBLIC_REST_URL`: The REST URL for the XION blockchain (default: https://api.xion-testnet-2.burnt.com)

## Development

The application is built with:
- Next.js
- React
- @burnt-labs/abstraxion for wallet integration
- Tailwind CSS for styling

## Getting Started

First, run the development server:

```bash
npm run dev
# or
yarn dev
# or
pnpm dev
# or
bun dev
```

Open [http://localhost:3000](http://localhost:3000) with your browser to see the result.

You can start editing the page by modifying `app/page.tsx`. The page auto-updates as you edit the file.

This project uses [`next/font`](https://nextjs.org/docs/basic-features/font-optimization) to automatically optimize and load Inter, a custom Google Font.

## Learn More

To learn more about Next.js, take a look at the following resources:

- [Next.js Documentation](https://nextjs.org/docs) - learn about Next.js features and API.
- [Learn Next.js](https://nextjs.org/learn) - an interactive Next.js tutorial.

You can check out [the Next.js GitHub repository](https://github.com/vercel/next.js/) - your feedback and contributions are welcome!

## Deploy on Vercel

The easiest way to deploy your Next.js app is to use the [Vercel Platform](https://vercel.com/new?utm_medium=default-template&filter=next.js&utm_source=create-next-app&utm_campaign=create-next-app-readme) from the creators of Next.js.

Check out our [Next.js deployment documentation](https://nextjs.org/docs/deployment) for more details.
