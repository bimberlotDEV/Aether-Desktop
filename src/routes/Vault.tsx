import { VaultView } from '@/components/vault/VaultView'
import { useVaultSpaces } from '@/hooks/useVault'

export function Vault() {
  const { spaces } = useVaultSpaces()
  return <VaultView spaces={spaces} />
}
