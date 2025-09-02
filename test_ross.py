import ross_link


def test_schedule():
    sched = ross_link.Schedule(["BA Physics", "BA Chemistry"], ["THEO-1100"])
    sched.validate()
    assert sched.is_valid()
    sched.display()
    print(sched.get_courses())
    print(sched.get_programs())
    sched.save("ross_pytest.xlsx")
    sched2 = ross_link.Schedule.from_file("ross_pytest.xlsx")
    assert sched2.is_valid()


def test_reasons():
    sched = ross_link.Schedule(["BA Physics", "BA Chemistry"], ["THEO-1100"])
    sched.validate()
    assert sched.is_valid()
    print(sched.get_reasons())


def test_course_reasons():
    sched = ross_link.Schedule(["BA Physics", "BA Chemistry"], ["THEO-1100"])
    sched.validate()
    assert sched.is_valid()

    def sub_test():
        for reasons in sched.get_reasons().values():
            for example in reasons:
                if example["type"] in ("ProgramElective", "Foundation", "SkillsAndPerspective", "Core"):
                    return example
        raise ValueError("No suitable example found")

    example = sub_test()
    print(example)
    print(
        sched.get_courses_for_reason(
            getattr(ross_link.ReasonTypes, example["type"]), name=example.get("name"), prog=example.get("prog")
        )
    )
