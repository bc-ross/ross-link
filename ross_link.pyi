class ReasonTypes:
    """An enum representing the different types of reasons a course may be included in a schedule."""

    Core: "ReasonTypes"
    Foundation: "ReasonTypes"
    SkillsAndPerspective: "ReasonTypes"
    ProgramRequired: "ReasonTypes"
    ProgramElective: "ReasonTypes"
    CourseReq: "ReasonTypes"

class Schedule:
    """A class to validate and create college schedules."""

    def __init__(self, programs: list[str], incoming: list[str] | None = None) -> None: ...
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

    def get_reasons(self) -> dict[str, list[dict[str, str]]]:
        """Get the reason(s) for every course's inclusion in the schedule."""

    def get_courses_for_reason(
        self, reason: "ReasonTypes", *, name: str | None = None, prog: str | None = None
    ) -> list[str]:
        """Get the list of courses which would satisfy a given reason."""
