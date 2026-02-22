import {
  Badge,
  Box,
  Grid,
  Heading,
  HStack,
  Text,
  VStack,
} from "@chakra-ui/react";

type TrackStat = {
  readonly trackName: string;
  readonly averagePosition: number;
  readonly racesPlayed: number;
};

type PlayerTrackStatsProps = {
  readonly trackStats: ReadonlyArray<TrackStat>;
};

const MIN_RACES = 2;

const getPositionColor = (avgPosition: number): string => {
  if (avgPosition <= 1.5) return "green.500";
  if (avgPosition <= 2.5) return "green.600";
  if (avgPosition <= 3.0) return "gray.700";
  if (avgPosition <= 3.5) return "orange.500";
  return "red.500";
};

const TrackEntry = ({
  stat,
  rank,
}: {
  readonly stat: TrackStat;
  readonly rank: number;
}) => (
  <Box
    p={{ base: 3, md: 4 }}
    bg="bg.panel"
    borderRadius="card"
    borderWidth="1px"
    borderColor="gray.200"
    boxShadow="card"
  >
    <HStack justify="space-between" gap={{ base: 2, md: 3 }}>
      <HStack gap={{ base: 2, md: 3 }} flex={1} minW={0}>
        <Badge
          colorScheme="gray"
          fontSize={{ base: "sm", md: "md" }}
          px={2}
          py={0.5}
          borderRadius="md"
          fontWeight="bold"
        >
          #{rank}
        </Badge>
        <VStack align="start" gap={0} minW={0}>
          <Text
            fontWeight="bold"
            fontSize={{ base: "sm", md: "md" }}
            color="gray.900"
            truncate
          >
            {stat.trackName}
          </Text>
          <Text fontSize={{ base: "xs", md: "sm" }} color="gray.600">
            {stat.racesPlayed} races
          </Text>
        </VStack>
      </HStack>
      <Text
        fontSize={{ base: "lg", md: "xl" }}
        fontWeight="bold"
        color={getPositionColor(stat.averagePosition)}
        flexShrink={0}
      >
        {stat.averagePosition.toFixed(1)}
      </Text>
    </HStack>
  </Box>
);

const TrackSection = ({
  title,
  tracks,
  startRank,
}: {
  readonly title: string;
  readonly tracks: ReadonlyArray<TrackStat>;
  readonly startRank: number;
}) => (
  <VStack gap={{ base: 2, md: 3 }} align="stretch">
    <Heading size={{ base: "sm", md: "md" }} color="gray.700">
      {title}
    </Heading>
    {tracks.map((stat, index) => (
      <TrackEntry key={stat.trackName} stat={stat} rank={startRank + index} />
    ))}
  </VStack>
);

export const PlayerTrackStats = ({ trackStats }: PlayerTrackStatsProps) => {
  const qualifying = trackStats.filter((s) => s.racesPlayed >= MIN_RACES);

  if (qualifying.length === 0) {
    return null;
  }

  const best = qualifying.slice(0, 3);
  const worst = qualifying.length > 3 ? qualifying.slice(-3) : [];

  return (
    <Grid
      templateColumns={{ base: "1fr", md: "1fr 1fr" }}
      gap={{ base: 4, md: 6 }}
    >
      <TrackSection title="Best Maps" tracks={best} startRank={1} />
      {worst.length > 0 && (
        <TrackSection
          title="Worst Maps"
          tracks={worst}
          startRank={qualifying.length - worst.length + 1}
        />
      )}
    </Grid>
  );
};
