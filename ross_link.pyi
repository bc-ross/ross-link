class Schedule:
    """A class to validate and create college schedules."""

    def __init__(self, programs: list[str]) -> None: ...
    @staticmethod
    def from_file(filename: str) -> "Schedule":
        """Load a schedule from an Excel file."""

    def is_valid(self) -> bool:
        """Check if the schedule is valid."""

    def save(self, filename: str) -> None:
        """Save the schedule to an Excel file."""

    def display(self) -> None:
        """Display the schedule in a human-readable format."""

    def get_courses(self) -> list[tuple[str, int | str, str]]:
        """Get the list of semesters & courses in the schedule."""

    @staticmethod
    def get_programs() -> list[str]:
        """Get the list of available programs."""
