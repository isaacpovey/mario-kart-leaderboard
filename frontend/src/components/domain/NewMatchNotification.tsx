import {
  Button,
  HStack,
  Text,
  VStack,
  DialogBackdrop,
  DialogBody,
  DialogCloseTrigger,
  DialogContent,
  DialogHeader,
  DialogRoot,
  DialogTitle,
} from "@chakra-ui/react";
import { useEffect, useState } from "react";
import { LuX } from "react-icons/lu";
import { useNavigate } from "react-router";

type Match = {
  id: string;
  time: string;
  completed: boolean;
};

type NewMatchNotificationProps = {
  matches: Match[];
  tournamentId: string | null;
};

export const NewMatchNotification = ({
  matches,
  tournamentId,
}: NewMatchNotificationProps) => {
  const navigate = useNavigate();
  const [isOpen, setIsOpen] = useState(false);
  const [newMatchId, setNewMatchId] = useState<string | null>(null);
  const [lastSeenMatchCount, setLastSeenMatchCount] = useState<number>(() => {
    if (!tournamentId) return 0;
    const stored = localStorage.getItem(`lastSeenMatchCount_${tournamentId}`);
    return stored ? Number.parseInt(stored, 10) : matches.length;
  });

  useEffect(() => {
    if (!tournamentId) return;

    const currentMatchCount = matches.length;

    if (currentMatchCount > lastSeenMatchCount) {
      const latestMatch = matches[matches.length - 1];
      if (latestMatch) {
        setNewMatchId(latestMatch.id);
        setIsOpen(true);

        const timer = setTimeout(() => {
          setIsOpen(false);
        }, 10000);

        return () => clearTimeout(timer);
      }
    }

    setLastSeenMatchCount(currentMatchCount);
    localStorage.setItem(
      `lastSeenMatchCount_${tournamentId}`,
      currentMatchCount.toString(),
    );
  }, [matches.length, tournamentId, lastSeenMatchCount, matches]);

  const handleViewMatch = () => {
    if (newMatchId) {
      navigate(`/match/${newMatchId}`);
      setIsOpen(false);
    }
  };

  const handleClose = () => {
    setIsOpen(false);
  };

  return (
    <DialogRoot open={isOpen} onOpenChange={(e) => setIsOpen(e.open)}>
      <DialogBackdrop />
      <DialogContent>
        <DialogHeader>
          <DialogTitle>New Race Created!</DialogTitle>
          <DialogCloseTrigger asChild>
            <Button variant="ghost" size="sm">
              <LuX />
            </Button>
          </DialogCloseTrigger>
        </DialogHeader>
        <DialogBody>
          <VStack gap={4} align="stretch">
            <Text>A new race has been created and is ready to play!</Text>
            <HStack gap={3} justify="flex-end">
              <Button onClick={handleClose} variant="outline">
                Dismiss
              </Button>
              <Button onClick={handleViewMatch} colorScheme="blue">
                View Race
              </Button>
            </HStack>
          </VStack>
        </DialogBody>
      </DialogContent>
    </DialogRoot>
  );
};
