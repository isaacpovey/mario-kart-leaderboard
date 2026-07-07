import { Box, HStack, Icon, Text, chakra } from '@chakra-ui/react'
import { Fragment } from 'react'
import type { ComponentType } from 'react'

export type BottomNavItem = {
  id: string
  label: string
  icon: ComponentType
  targetId?: string
  onClick?: () => void
  dividerAfter?: boolean
}

const scrollToSection = (targetId: string) => {
  const el = document.getElementById(targetId)
  if (!el) {
    return
  }
  el.scrollIntoView({ behavior: 'smooth', block: 'start' })
}

const handleItemClick = (item: BottomNavItem) => {
  if (item.onClick) {
    item.onClick()
    return
  }
  if (item.targetId) {
    scrollToSection(item.targetId)
  }
}

type BottomNavProps = {
  items: BottomNavItem[]
}

export const BottomNav = ({ items }: BottomNavProps) => {
  if (items.length === 0) {
    return null
  }

  return (
    <Box
      position="fixed"
      bottom={0}
      left={0}
      right={0}
      bg="bg.panel"
      borderTopWidth="1px"
      borderColor="gray.200"
      boxShadow="0 -2px 8px rgba(0,0,0,0.06)"
      zIndex={10}
      pb="env(safe-area-inset-bottom)"
    >
      <HStack as="nav" aria-label="Section navigation" justify="space-around" align="stretch" maxW="4xl" mx="auto" px={2} py={2} gap={0}>
        {items.map((item) => (
          <Fragment key={item.id}>
            <chakra.button
              type="button"
              onClick={() => handleItemClick(item)}
              display="flex"
              flexDirection="column"
              alignItems="center"
              justifyContent="center"
              flex={1}
              gap={1}
              py={2}
              px={1}
              mx={1}
              borderRadius="button"
              color="gray.700"
              bg="transparent"
              cursor="pointer"
              transition="background-color 0.15s, color 0.15s"
              _hover={{ bg: 'gray.100', color: 'brand.600' }}
              _active={{ bg: 'gray.200' }}
              _focusVisible={{ outline: '2px solid', outlineColor: 'brand.500', outlineOffset: '2px' }}
            >
              <Icon as={item.icon} boxSize={5} />
              <Text fontSize="xs" fontWeight="medium">
                {item.label}
              </Text>
            </chakra.button>
            {item.dividerAfter && <Box alignSelf="center" w="1px" h="60%" bg="gray.200" />}
          </Fragment>
        ))}
      </HStack>
    </Box>
  )
}
