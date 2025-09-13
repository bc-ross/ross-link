from __future__ import annotations

from .ross_link import *

__doc__ = ross_link.__doc__
if hasattr(ross_link, "__all__"):
    __all__ = ross_link.__all__


class Schedule(ross_link.Schedule):
    @staticmethod
    def with_courses(
        programs: list[str], incoming: list[str] | None = None, courses: dict[str, list[str]] | None = None
    ) -> Schedule:
        semesters = {}
        summers = {}
        non_term = None
        incoming = incoming or []
        if courses is not None:
            for course in courses:
                if "semester-" in course:
                    semesters[course[len("semester-") :]] = courses[course]
                if "summer-" in course:
                    summers[course[len("summer-") :]] = courses[course]
                if "non-term" == course:
                    non_term = courses[course]
        semesters_sorted = list(dict(sorted(semesters.items())).values())
        incoming += non_term or []
        return ross_link.Schedule._with_courses(programs, incoming, semesters_sorted)
