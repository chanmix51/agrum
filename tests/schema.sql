--
-- PostgreSQL database dump
--

\restrict h1sPrhU4JhzqJ6EaTXoO38KUuSd7huNaGT1yqVLGPII8emXqeg1Af98l7RjwYPY

-- Dumped from database version 18.0 (Debian 18.0-1.pgdg13+3)
-- Dumped by pg_dump version 18.1 (Ubuntu 18.1-1.pgdg24.04+2)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: pommr; Type: SCHEMA; Schema: -; Owner: -
--

CREATE SCHEMA pommr;


SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: address; Type: TABLE; Schema: pommr; Owner: -
--

CREATE TABLE pommr.address (
    address_id uuid DEFAULT uuidv4() NOT NULL,
    label text NOT NULL,
    company_id uuid NOT NULL,
    content text NOT NULL,
    zipcode text NOT NULL,
    city text NOT NULL,
    associated_contact_id uuid
);


--
-- Name: company; Type: TABLE; Schema: pommr; Owner: -
--

CREATE TABLE pommr.company (
    company_id uuid DEFAULT uuidv4() NOT NULL,
    name text NOT NULL,
    default_address_id uuid NOT NULL
);


--
-- Name: contact; Type: TABLE; Schema: pommr; Owner: -
--

CREATE TABLE pommr.contact (
    contact_id uuid DEFAULT uuidv4() NOT NULL,
    name text NOT NULL,
    email text,
    phone_number text,
    company_id uuid NOT NULL
);


--
-- Name: address address_pkey; Type: CONSTRAINT; Schema: pommr; Owner: -
--

ALTER TABLE ONLY pommr.address
    ADD CONSTRAINT address_pkey PRIMARY KEY (address_id);


--
-- Name: company company_pkey; Type: CONSTRAINT; Schema: pommr; Owner: -
--

ALTER TABLE ONLY pommr.company
    ADD CONSTRAINT company_pkey PRIMARY KEY (company_id);


--
-- Name: contact contact_pkey; Type: CONSTRAINT; Schema: pommr; Owner: -
--

ALTER TABLE ONLY pommr.contact
    ADD CONSTRAINT contact_pkey PRIMARY KEY (contact_id);


--
-- Name: address address_have_one_default_contact; Type: FK CONSTRAINT; Schema: pommr; Owner: -
--

ALTER TABLE ONLY pommr.address
    ADD CONSTRAINT address_have_one_default_contact FOREIGN KEY (associated_contact_id) REFERENCES pommr.contact(contact_id);


--
-- Name: CONSTRAINT address_have_one_default_contact ON address; Type: COMMENT; Schema: pommr; Owner: -
--

COMMENT ON CONSTRAINT address_have_one_default_contact ON pommr.address IS 'Address may have one default contact. Having a default contact prevents the contact from being deleted (customer requirement see #3456)';


--
-- Name: company company_default_address; Type: FK CONSTRAINT; Schema: pommr; Owner: -
--

ALTER TABLE ONLY pommr.company
    ADD CONSTRAINT company_default_address FOREIGN KEY (default_address_id) REFERENCES pommr.address(address_id) ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED;


--
-- Name: address each_addresse_belongs_to_one_company; Type: FK CONSTRAINT; Schema: pommr; Owner: -
--

ALTER TABLE ONLY pommr.address
    ADD CONSTRAINT each_addresse_belongs_to_one_company FOREIGN KEY (company_id) REFERENCES pommr.company(company_id) ON DELETE CASCADE;


--
-- Name: CONSTRAINT each_addresse_belongs_to_one_company ON address; Type: COMMENT; Schema: pommr; Owner: -
--

COMMENT ON CONSTRAINT each_addresse_belongs_to_one_company ON pommr.address IS 'Each address belongs to a single company. If the company is dropped, all associated addresses are dropped.';


--
-- Name: contact each_contact_belongs_to_one_company; Type: FK CONSTRAINT; Schema: pommr; Owner: -
--

ALTER TABLE ONLY pommr.contact
    ADD CONSTRAINT each_contact_belongs_to_one_company FOREIGN KEY (company_id) REFERENCES pommr.company(company_id) ON DELETE CASCADE;


--
-- Name: CONSTRAINT each_contact_belongs_to_one_company ON contact; Type: COMMENT; Schema: pommr; Owner: -
--

COMMENT ON CONSTRAINT each_contact_belongs_to_one_company ON pommr.contact IS 'Each contact belongs to a single company. If the company is dropped, all associated contacts are dropped.';


--
-- PostgreSQL database dump complete
--

\unrestrict h1sPrhU4JhzqJ6EaTXoO38KUuSd7huNaGT1yqVLGPII8emXqeg1Af98l7RjwYPY

