import { Circle, Image } from '@chakra-ui/react'
import { useState } from 'react'

type AvatarProps = {
  name: string
  avatarFilename?: string | null
  size?: 'sm' | 'md' | 'lg'
}

const sizeMap = {
  sm: { box: '32px', text: 'sm' },
  md: { box: '40px', text: 'md' },
  lg: { box: '56px', text: 'lg' },
}

const colorMap = ['red.500', 'orange.500', 'yellow.500', 'green.500', 'teal.500', 'blue.500', 'cyan.500', 'purple.500', 'pink.500']

const getInitials = (name: string): string => {
  const parts = name.trim().split(' ')
  if (parts.length === 1) {
    return parts[0].substring(0, 2).toUpperCase()
  }
  return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase()
}

const getColorForName = (name: string): string => {
  const hash = name.split('').reduce((acc, char) => acc + char.charCodeAt(0), 0)
  return colorMap[hash % colorMap.length]
}

export const Avatar = ({ name, avatarFilename, size = 'md' }: AvatarProps) => {
  const [imageError, setImageError] = useState(false)
  const initials = getInitials(name)
  const bgColor = getColorForName(name)
  const dimensions = sizeMap[size]

  const shouldShowImage = avatarFilename && !imageError

  return (
    <Circle size={dimensions.box} bg={bgColor} color="white" fontWeight="bold" fontSize={dimensions.text} flexShrink={0} overflow="hidden">
      {shouldShowImage ? (
        <Image
          src={`/avatars/${avatarFilename}`}
          alt={name}
          width="100%"
          height="100%"
          objectFit="cover"
          onError={() => setImageError(true)}
        />
      ) : (
        initials
      )}
    </Circle>
  )
}
