Requirements Structure
======================

Repeater Atlas requirements are stored as small, standalone
reStructuredText files and assembled into larger documents for review,
publishing, and Wiki output. The model is inspired by ReqIF, but
intentionally keeps the repository format simple and practical.

Model
-----

Each requirement is a single object, stored in one file under
``docs/requirements/``. The directory is flat: requirements are not
organized by subdirectory. Chapter structure is created by documents
that include the requirement files in the desired order.

Requirement Files
-----------------

Each requirement file must be readable by itself and useful when
included in a larger document. It must contain exactly one requirement.

Requirement files live directly under ``docs/requirements/`` and are
named after their requirement ID in lowercase. Requirement IDs are a
plain sequence starting at ``RA-1000``. Always use the next available
number when creating new requirements.

The assembled requirements document lives at
``docs/requirements/README.rst``. It includes individual requirement
files in chapter order while keeping the requirement files themselves in
a single flat directory.

Example filenames:

- ``docs/requirements/ra-1000.rst``
- ``docs/requirements/ra-1001.rst``
- ``docs/requirements/ra-1002.rst``

Requirement Template
--------------------

Use this structure for new requirements:

::

   .. requirement::
      :id: RA-1000
      :type: Functional
      :status: Draft
      :verification: HTTP integration test

   RA-1000: Password Reset Request
   -------------------------------

   The application shall let a user request a password reset email for an
   existing account.

   Rationale
   ~~~~~~~~~

   Users need a way to recover access without administrator assistance.

   Acceptance Criteria
   ~~~~~~~~~~~~~~~~~~~

   * A user can submit an email address from the password reset form.
   * If the email address belongs to an account, the application sends a password
     reset email.
   * The response does not reveal whether the submitted email address belongs to
     an account.

   Notes
   ~~~~~

   This requirement covers requesting a reset email only. Choosing a new password
   after following the reset link is a separate requirement.

   Links
   ~~~~~

   * Related: ``RA-1001``

Heading Levels
--------------

Requirement files use heading levels that compose cleanly when included
into a larger document:

- Requirement title: ``-----`` underline.
- Requirement sections: ``~~~~~`` underline.

Do not use top-level ``=====`` headings inside individual requirement
files. Those are reserved for assembled documents and chapters.

Metadata
--------

The ``.. requirement::`` directive is intended for Pandoc and Lua
filters:

- ``id``: required stable identifier, for example ``RA-1000``.
- ``title``: required human-readable title used with the ID to generate the
  requirement heading.
- ``type``: requirement type. Allowed values are ``Functional``,
  ``Non-Functional``, ``Constraint``, ``Interface``, ``Data``, and
  ``UX``.
- ``status``: lifecycle state. Allowed values are ``Draft``,
  ``Proposed``, ``Accepted``, ``Implemented``, and ``Deprecated``.
- ``verification``: how the requirement can be checked.

If a requirement needs traceability to a design document, ticket, test,
or external source, put that in the ``Links`` section.

Chapter Documents
-----------------

Chapters assemble requirement files and provide explanatory prose. They
may be maintained as ``README.rst`` files or generated from another
source, depending on the publishing flow.

Example chapter file:

::

   Account Access Requirements
   ===========================

   These requirements describe how users access and recover their accounts.

   Core Behavior
   -------------

   .. include:: ra-1000.rst

   .. include:: ra-1001.rst

   Recovery Behavior
   -----------------

   These requirements describe account recovery flows.

   .. include:: ra-1002.rst

The same requirement file may appear in more than one assembled view
when that is useful, but the requirement file remains the canonical
source.

Authoring Rules
---------------

- One requirement per file.
- Requirement IDs are plain sequential IDs starting at ``RA-1000``.
- Keep the normative requirement sentence short and testable.
- Put explanation in ``Rationale``, not in the requirement sentence.
- Use ``Acceptance Criteria`` for concrete observable behavior.
- Use ``Links`` for traceability to design documents, tickets, tests, or
  other requirements.
